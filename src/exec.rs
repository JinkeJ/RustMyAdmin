use std::sync::Arc;

use serde_json::json;
use sqlx::Row;

use crate::error::ApiResult;
use crate::session::Session;
use crate::sqlutil::{is_binary_type, returns_rows, row_cells, row_columns, Cell};

/// 单条语句的执行结果
#[derive(Debug)]
pub enum StmtResult {
    Rows {
        columns: Vec<String>,
        rows: Vec<Vec<Cell>>,
        truncated: bool,
    },
    Affected {
        rows: u64,
        last_insert_id: u64,
    },
}

const MAX_CONSOLE_ROWS: usize = 1000;

/// 执行单条 SQL：
/// - 返回结果集的语句：raw_sql 文本协议取行（值均为精确文本，二进制列转 hex）
/// - 空结果集时通过 prepare 获取列名，保证表头仍可显示
/// - 其余语句（DDL/USE/SET 等）：raw_sql 执行并返回影响行数
pub async fn exec_statement(sess: &Arc<Session>, stmt: &str) -> ApiResult<StmtResult> {
    if returns_rows(stmt) {
        let stmt_owned = stmt.to_string();
        let rows = sess.raw_rows(stmt_owned).await?;
        let truncated = rows.len() > MAX_CONSOLE_ROWS;

        let columns = if !rows.is_empty() {
            row_columns(&rows[0])
        } else {
            // 空结果集：prepare 拿列名
            sess.describe_columns(stmt.to_string()).await.unwrap_or_default()
        };

        // 控制台无列类型元信息，非 UTF-8 字节由解码层回退为 hex 显示
        let out: Vec<Vec<Cell>> = rows
            .into_iter()
            .take(MAX_CONSOLE_ROWS)
            .map(|r| row_cells(&r, None))
            .collect();

        Ok(StmtResult::Rows { columns, rows: out, truncated })
    } else {
        let (rows, last_insert_id) = sess.raw_exec(stmt).await?;
        Ok(StmtResult::Affected { rows, last_insert_id })
    }
}

/// 把执行结果序列化为前端 JSON
pub fn stmt_result_json(r: &StmtResult, ms: u128, sql: &str) -> serde_json::Value {
    let mut o = match r {
        StmtResult::Rows { columns, rows, truncated } => {
            json!({
                "kind": "rows",
                "columns": columns,
                "rows": rows.iter().map(|row| {
                    row.iter().map(|c| c.display()).collect::<Vec<_>>()
                }).collect::<Vec<_>>(),
                "rowCount": rows.len(),
                "truncated": truncated,
            })
        }
        StmtResult::Affected { rows, last_insert_id } => {
            json!({
                "kind": "affected",
                "affectedRows": rows,
                "lastInsertId": last_insert_id,
            })
        }
    };
    o["ms"] = json!(ms);
    o["sql"] = json!(sql);
    o
}

/// 批量执行一段 SQL 文本（控制台/导入共用），默认遇错停止
pub async fn exec_batch(
    sess: &Arc<Session>,
    sql_text: &str,
    continue_on_error: bool,
) -> ApiResult<serde_json::Value> {
    let stmts = crate::sqlutil::split_sql(sql_text);
    let total = stmts.len();
    let mut results = Vec::new();
    let mut executed = 0usize;
    let mut error: Option<serde_json::Value> = None;

    for (idx, stmt) in stmts.iter().enumerate() {
        let t0 = std::time::Instant::now();
        match exec_statement(sess, stmt).await {
            Ok(r) => {
                executed += 1;
                if results.len() < 200 {
                    results.push(stmt_result_json(&r, t0.elapsed().as_millis(), stmt));
                }
            }
            Err(e) => {
                error = Some(json!({
                    "index": idx,
                    "statement": stmt,
                    "message": e.message,
                }));
                if !continue_on_error {
                    break;
                }
            }
        }
    }

    Ok(json!({
        "statements": total,
        "executed": executed,
        "results": results,
        "error": error,
    }))
}

/// 获取表的列元信息（名称、数据类型、二进制标记等）
pub struct ColumnMeta {
    pub name: String,
    pub data_type: String,
    pub column_type: String,
    pub nullable: bool,
    pub key: String,
    pub extra: String,
    pub default: Option<String>,
    pub collation: Option<String>,
}

pub async fn column_meta(sess: &Arc<Session>, db: &str, table: &str) -> ApiResult<Vec<ColumnMeta>> {
    let sql = format!(
        "SELECT COLUMN_NAME, DATA_TYPE, COLUMN_TYPE, IS_NULLABLE, COLUMN_KEY, EXTRA, COLUMN_DEFAULT, COLLATION_NAME \
         FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = {} AND TABLE_NAME = {} ORDER BY ORDINAL_POSITION",
        crate::sqlutil::sql_string_lit(db),
        crate::sqlutil::sql_string_lit(table)
    );
    let rows = sess.raw_rows(sql).await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let get = |i: usize| -> ApiResult<Option<String>> { Ok(r.try_get_unchecked::<Option<String>, usize>(i)?) };
        out.push(ColumnMeta {
            name: get(0)?.unwrap_or_default(),
            data_type: get(1)?.unwrap_or_default(),
            column_type: get(2)?.unwrap_or_default(),
            nullable: get(3)?.as_deref() == Some("YES"),
            key: get(4)?.unwrap_or_default(),
            extra: get(5)?.unwrap_or_default(),
            default: get(6)?,
            collation: get(7)?,
        });
    }
    Ok(out)
}

/// 获取主键列（按顺序），无主键返回空
pub async fn primary_key(sess: &Arc<Session>, db: &str, table: &str) -> ApiResult<Vec<String>> {
    let sql = format!(
        "SELECT COLUMN_NAME FROM information_schema.KEY_COLUMN_USAGE \
         WHERE TABLE_SCHEMA = {} AND TABLE_NAME = {} AND CONSTRAINT_NAME = 'PRIMARY' \
         ORDER BY ORDINAL_POSITION",
        crate::sqlutil::sql_string_lit(db),
        crate::sqlutil::sql_string_lit(table)
    );
    let rows = sess.raw_rows(sql).await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push(r.try_get_unchecked::<String, usize>(0)?);
    }
    Ok(out)
}

/// 表列的二进制标记（与列顺序一一对应）
pub fn binary_flags_of(cols: &[ColumnMeta]) -> Vec<bool> {
    cols.iter().map(|c| is_binary_type(&c.data_type)).collect()
}
