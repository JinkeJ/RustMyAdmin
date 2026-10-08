use std::io::Cursor;
use std::io::{Read as _, Write};
use std::sync::Arc;

use axum::extract::{Multipart, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;

use crate::error::{ApiError, ApiResult};
use crate::exec::{binary_flags_of, column_meta, exec_batch};
use crate::session::{AppState, Session};
use crate::sqlutil::{check_ident, ident, qualify, row_cells, row_columns, sql_string_lit, Cell};

// ============ SQL console ============

#[derive(Deserialize)]
pub struct SqlReq {
    pub sql: String,
    #[serde(default)]
    pub db: Option<String>,
    #[serde(default)]
    pub continue_on_error: bool,
}

pub async fn exec_sql(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<SqlReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    if req.sql.trim().is_empty() {
        return Err(ApiError::bad("SQL 不能为空"));
    }

    // If a target database is specified, USE it first (single-connection session ensures it applies to subsequent statements)
    if let Some(db) = &req.db {
        if !db.is_empty() {
            check_ident(db, "数据库")?;
            sess.raw_exec_ok(format!("USE {}", ident(db))).await?;
        }
    }

    let result = exec_batch(&sess, &req.sql, req.continue_on_error).await?;
    Ok(Json(result))
}

// ============ Export ============

#[derive(Deserialize)]
pub struct ExportQuery {
    pub db: String,
    /// Comma-separated table name list; defaults to all tables
    pub tables: Option<String>,
    /// Export mode: full (default) / structure (schema only) / data (data only)
    pub mode: Option<String>,
}

/// Generate mysqldump-style SQL dump text for the whole database or selected tables
async fn build_dump(
    sess: &Arc<Session>,
    db: &str,
    tables: Option<&str>,
    mode: &str,
) -> ApiResult<String> {
    let with_structure = mode == "full" || mode == "structure";
    let with_data = mode == "full" || mode == "data";
    let mut out = String::with_capacity(1 << 20);
    out.push_str("-- RustMyAdmin SQL dump\n");
    out.push_str(&format!("-- 主机: {}\n", sess.host));
    out.push_str(&format!(
        "-- 生成时间: {}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    out.push_str(&format!("-- 导出模式: {mode}\n"));
    out.push_str("SET NAMES utf8mb4;\n");
    out.push_str("SET FOREIGN_KEY_CHECKS = 0;\n");
    out.push_str("SET SQL_MODE = 'NO_AUTO_VALUE_ON_ZERO';\n");
    out.push_str("SET TIME_ZONE = '+00:00';\n\n");
    out.push_str(&format!(
        "CREATE DATABASE IF NOT EXISTS {} DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;\n",
        ident(db)
    ));
    out.push_str(&format!("USE {};\n\n", ident(db)));

    // Table list (including views)
    let rows = sess
        .raw_rows(format!(
            "SELECT TABLE_NAME, TABLE_TYPE FROM information_schema.TABLES WHERE TABLE_SCHEMA = {} ORDER BY TABLE_TYPE, TABLE_NAME",
            sql_string_lit(db)
        ))
        .await?;
    let mut selected: Vec<(String, String)> = Vec::new();
    for r in rows {
        let name: String = r.try_get_unchecked::<String, usize>(0)?;
        let ty: String = r.try_get_unchecked::<String, usize>(1)?;
        if let Some(list) = tables {
            let names: Vec<&str> = list.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            if !names.contains(&name.as_str()) {
                continue;
            }
        }
        selected.push((name, ty));
    }
    if selected.is_empty() {
        return Err(ApiError::bad("没有可导出的表"));
    }

    const CHUNK_ROWS: usize = 250;
    let mut dumped_bytes = 0usize;
    for (name, ty) in &selected {
        let qt = qualify(db, name);
        let is_view = ty == "VIEW";
        out.push_str("\n-- ------------------------------------------------------------\n");
        out.push_str(&format!("-- 表: {}\n", name));
        out.push_str("-- ------------------------------------------------------------\n\n");

        if with_structure {
            if is_view {
                out.push_str(&format!("DROP VIEW IF EXISTS {};\n", qt));
            } else {
                out.push_str(&format!("DROP TABLE IF EXISTS {};\n", qt));
            }

            let create_sql = if is_view {
                format!("SHOW CREATE VIEW {qt}")
            } else {
                format!("SHOW CREATE TABLE {qt}")
            };
            let crows = sess.raw_rows(create_sql).await?;
            if let Some(r) = crows.first() {
                let ddl: String = r.try_get_unchecked::<String, usize>(1)?;
                out.push_str(&ddl);
                if !ddl.ends_with(';') {
                    out.push(';');
                }
                out.push_str("\n\n");
            }
        }

        if !is_view && with_data {
            // Column binary flags (bit/blob etc. exported as 0x hex)
            let cols = column_meta(sess, db, name).await?;
            let flags = binary_flags_of(&cols);

            out.push_str(&format!("\n-- 数据: {}\n", name));
            let data_rows = sess
                .raw_rows(format!("SELECT * FROM {qt}"))
                .await?;
            if data_rows.is_empty() {
                out.push_str("-- 无数据\n\n");
                continue;
            }
            let col_names: Vec<String> = row_columns(&data_rows[0]);
            let mut pending = 0usize;
            for r in &data_rows {
                if pending == 0 {
                    out.push_str(&format!(
                        "INSERT INTO {} ({}) VALUES\n",
                        qt,
                        col_names.iter().map(|c| ident(c)).collect::<Vec<_>>().join(", ")
                    ));
                }
                let cells = row_cells(r, Some(&flags));
                let vals: Vec<String> = cells
                    .iter()
                    .map(|cl| match cl {
                        Cell::N => "NULL".to_string(),
                        Cell::S(s) => sql_string_lit(s),
                        Cell::B(b) => {
                            let mut h = String::with_capacity(b.len() * 2);
                            for byte in b {
                                h.push_str(&format!("{byte:02X}"));
                            }
                            format!("0x{h}")
                        }
                    })
                    .collect();
                out.push_str("(");
                out.push_str(&vals.join(", "));
                out.push_str(")");
                pending += 1;
                if pending >= CHUNK_ROWS {
                    out.push_str(";\n");
                    pending = 0;
                } else {
                    out.push_str(",\n");
                }
            }
            if pending > 0 {
                if out.ends_with(",\n") {
                    out.truncate(out.len() - 2);
                }
                out.push_str(";\n");
            }
            out.push('\n');

            dumped_bytes += out.len();
            if dumped_bytes > 512 * 1024 * 1024 {
                return Err(ApiError::bad("导出内容过大（超过 512MB），请分表导出"));
            }
        }
    }

    out.push_str("\nSET FOREIGN_KEY_CHECKS = 1;\n");
    Ok(out)
}

pub async fn export(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<ExportQuery>,
) -> ApiResult<Response> {
    let sess = state.auth(&headers).await?;
    check_ident(&q.db, "数据库")?;
    let mode = q.mode.as_deref().map(str::trim).filter(|s| !s.is_empty()).unwrap_or("full");
    if !matches!(mode, "full" | "structure" | "data") {
        return Err(ApiError::bad("导出模式无效"));
    }
    let sql = build_dump(&sess, &q.db, q.tables.as_deref(), mode).await?;

    // Pack into zip
    let mut buf = Cursor::new(Vec::with_capacity(sql.len() / 2 + 1024));
    {
        let mut zw = zip::ZipWriter::new(&mut buf);
        let opts: zip::write::SimpleFileOptions = Default::default();
        zw.start_file(format!("{}.sql", q.db), opts)
            .map_err(|e| ApiError::internal(format!("写入 zip 失败: {e}")))?;
        zw.write_all(sql.as_bytes())
            .map_err(|e| ApiError::internal(format!("写入 zip 失败: {e}")))?;
        zw.finish()
            .map_err(|e| ApiError::internal(format!("完成 zip 失败: {e}")))?;
    }
    let bytes = buf.into_inner();
    let filename = format!("{}.sql.zip", q.db);

    Ok((
        [
            ("content-type", "application/zip".to_string()),
            (
                "content-disposition",
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        bytes,
    )
        .into_response())
}

// ============ Import ============

pub async fn import(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;

    let mut sql_text: Option<String> = None;
    let mut filename = String::new();
    let mut target_db: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::bad(format!("读取上传失败: {e}")))?
    {
        match field.name().unwrap_or("") {
            "file" => {
                filename = field.file_name().unwrap_or("upload").to_string();
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| ApiError::bad(format!("读取上传文件失败: {e}")))?;
                let lower = filename.to_lowercase();
                let is_zip = lower.ends_with(".zip")
                    || (data.len() > 4 && data[0] == b'P' && data[1] == b'K');
                if is_zip {
                    let mut za = zip::ZipArchive::new(Cursor::new(&data[..]))
                        .map_err(|e| ApiError::bad(format!("打开 zip 失败: {e}")))?;
                    let mut found: Option<usize> = None;
                    for i in 0..za.len() {
                        let entry = za
                            .by_index(i)
                            .map_err(|e| ApiError::bad(format!("读取 zip 条目失败: {e}")))?;
                        if entry.name().to_lowercase().ends_with(".sql") {
                            found = Some(i);
                            break;
                        }
                    }
                    let idx =
                        found.ok_or_else(|| ApiError::bad("zip 中未找到 .sql 文件"))?;
                    let mut entry = za
                        .by_index(idx)
                        .map_err(|e| ApiError::bad(format!("读取 zip 条目失败: {e}")))?;
                    let mut content = String::new();
                    entry
                        .read_to_string(&mut content)
                        .map_err(|e| ApiError::bad(format!("解压失败: {e}")))?;
                    sql_text = Some(content);
                } else {
                    sql_text = Some(String::from_utf8_lossy(&data).into_owned());
                }
            }
            "target_db" => {
                let v = field.text().await.unwrap_or_default();
                if !v.trim().is_empty() {
                    target_db = Some(v.trim().to_string());
                }
            }
            _ => {}
        }
    }

    let mut sql_text = sql_text.ok_or_else(|| ApiError::bad("未收到文件"))?;
    if let Some(db) = &target_db {
        check_ident(db, "目标数据库")?;
        sql_text = format!(
            "CREATE DATABASE IF NOT EXISTS {} DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci;\nUSE {};\n{}",
            ident(db),
            ident(db),
            sql_text
        );
    }

    let result = exec_batch(&sess, &sql_text, false).await?;
    Ok(Json(json!({ "file": filename, "result": result })))
}
