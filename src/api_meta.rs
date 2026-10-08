use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;
use sqlx::mysql::MySqlConnectOptions;

use crate::error::{ApiError, ApiResult};
use crate::exec::{column_meta, primary_key};
use crate::session::{AppState, Session, session_pool};
use crate::sqlutil::{check_ident, ident, sql_string_lit};

// ============ Login / Session ============

/// Connect and probe server info: standalone async fn, owned args, queries via &pool
/// (avoids the sqlx connection borrow chain triggering a higher-ranked inference bug at the Handler dyn+Send coercion).
async fn login_connect(
    opts: MySqlConnectOptions,
) -> sqlx::Result<(sqlx::mysql::MySqlPool, String, String, String)> {
    let pool = session_pool(opts).await?;
    let info = sqlx::raw_sql("SELECT USER(), VERSION(), @@version_comment")
        .fetch_one(&pool)
        .await?;
    let full_user: String = info.try_get_unchecked::<String, usize>(0)?;
    let version: String = info.try_get_unchecked::<String, usize>(1)?;
    let comment: String = info.try_get_unchecked::<String, usize>(2)?;
    Ok((pool, full_user, version, comment))
}

#[derive(Deserialize)]
pub struct LoginReq {
    pub host: String,
    pub port: Option<u16>,
    pub user: String,
    pub password: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginReq>,
) -> ApiResult<Json<serde_json::Value>> {
    if req.user.is_empty() || req.host.is_empty() {
        return Err(ApiError::bad("主机和用户名不能为空"));
    }
    let port = req.port.unwrap_or(3306);
    let mut opts = MySqlConnectOptions::new()
        .host(&req.host)
        .port(port)
        .username(&req.user);
    // Empty password = passwordless auth: Some("") makes the client send an empty password hash,
    // which passwordless accounts reject (1045 using password: YES), so set it only when non-empty
    if !req.password.is_empty() {
        opts = opts.password(&req.password);
    }

    let (pool, full_user, version, comment) = login_connect(opts.clone())
        .await
        .map_err(|e| ApiError::connection_lost(format!("连接数据库失败: {e}")))?;

    let sess = Session::new(
        req.host.clone(),
        format!("{}@{}", req.user, req.host),
        pool,
    );
    let token = state.insert(sess).await;
    Ok(Json(json!({
        "token": token,
        "user": full_user,
        "loginUser": req.user,
        "host": req.host,
        "port": port,
        "version": version,
        "versionComment": comment,
    })))
}

pub async fn logout(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    if let Some(token) = headers.get("x-auth-token").and_then(|v| v.to_str().ok()) {
        state.remove(token).await;
    }
    Ok(Json(json!({ "ok": true })))
}

pub async fn status(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    let rows = sess
        .raw_rows("SELECT USER(), DATABASE(), VERSION(), @@version_comment, NOW()")
        .await?;
    let r = rows
        .first()
        .ok_or_else(|| ApiError::internal("无法读取服务器状态"))?;
    Ok(Json(json!({
        "user": r.try_get_unchecked::<String, usize>(0)?,
        "database": r.try_get_unchecked::<Option<String>, usize>(1)?,
        "version": r.try_get_unchecked::<String, usize>(2)?,
        "versionComment": r.try_get_unchecked::<String, usize>(3)?,
        "serverTime": r.try_get_unchecked::<String, usize>(4)?,
        "display": sess.display_name,
    })))
}

// ============ Database list / create / drop / rename ============

pub async fn databases(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    let rows = sess.raw_rows("SHOW DATABASES").await?;
    let mut names = Vec::with_capacity(rows.len());
    for r in rows {
        names.push(r.try_get_unchecked::<String, usize>(0)?);
    }
    Ok(Json(json!({ "databases": names })))
}

#[derive(Deserialize)]
pub struct CreateDbReq {
    pub name: String,
}

pub async fn create_database(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateDbReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&req.name, "数据库")?;
    sess.raw_exec_ok(format!(
        "CREATE DATABASE IF NOT EXISTS {} DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci",
        ident(&req.name)
    ))
    .await?;
    Ok(Json(json!({ "ok": true, "message": format!("数据库 {} 已创建", req.name) })))
}

pub async fn drop_database(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(db): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    sess.raw_exec_ok(format!("DROP DATABASE IF EXISTS {}", ident(&db)))
        .await?;
    Ok(Json(json!({ "ok": true, "message": format!("数据库 {} 已删除", db) })))
}

#[derive(Deserialize)]
pub struct RenameDbReq {
    pub to: String,
}

/// Database rename: MySQL has no native RENAME DATABASE; implemented as create new db + RENAME TABLE + drop old db
pub async fn rename_database(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(db): Path<String>,
    Json(req): Json<RenameDbReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "原数据库")?;
    check_ident(&req.to, "新数据库")?;
    if db == req.to {
        return Err(ApiError::bad("新名称与原名称相同"));
    }

    // create the new database (fixed utf8mb4_general_ci)
    sess.raw_exec_ok(format!(
        "CREATE DATABASE IF NOT EXISTS {} DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci",
        ident(&req.to)
    ))
    .await?;

    // fetch all tables (including views)
    let rows = sess
        .raw_rows(format!(
            "SELECT TABLE_NAME FROM information_schema.TABLES WHERE TABLE_SCHEMA = {}",
            sql_string_lit(&db)
        ))
        .await?;
    let tables: Vec<String> = {
        let mut v = Vec::with_capacity(rows.len());
        for r in rows {
            v.push(r.try_get_unchecked::<String, usize>(0)?);
        }
        v
    };

    if !tables.is_empty() {
        let mut parts = Vec::with_capacity(tables.len());
        for t in &tables {
            parts.push(format!(
                "{} TO {}",
                crate::sqlutil::qualify(&db, t),
                crate::sqlutil::qualify(&req.to, t)
            ));
        }
        let sql = format!("RENAME TABLE {}", parts.join(", "));
        if let Err(e) = sess.raw_exec_ok(sql).await {
            // on failure, try to clean up the already-created empty database
            let _ = sess
                .raw_exec_ok(format!("DROP DATABASE IF EXISTS {}", ident(&req.to)))
                .await;
            return Err(e);
        }
    }

    // drop the old database
    sess.raw_exec_ok(format!("DROP DATABASE IF EXISTS {}", ident(&db)))
        .await?;

    Ok(Json(json!({ "ok": true, "message": format!("数据库已重命名为 {}", req.to) })))
}

// ============ Table list / structure ============

pub async fn tables_overview(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(db): Path<String>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    let rows = sess
        .raw_rows(format!(
            "SELECT TABLE_NAME, TABLE_TYPE, ENGINE, TABLE_ROWS, (DATA_LENGTH + INDEX_LENGTH), TABLE_COLLATION \
             FROM information_schema.TABLES WHERE TABLE_SCHEMA = {} ORDER BY TABLE_NAME",
            sql_string_lit(&db)
        ))
        .await?;
    let mut tables = Vec::with_capacity(rows.len());
    for r in rows {
        let g = |i: usize| -> ApiResult<Option<String>> { Ok(r.try_get_unchecked::<Option<String>, usize>(i)?) };
        tables.push(json!({
            "name": g(0)?.unwrap_or_default(),
            "type": g(1)?.unwrap_or_default(),
            "engine": g(2)?,
            "rows": g(3)?.and_then(|s| s.parse::<i64>().ok()),
            "sizeBytes": g(4)?.and_then(|s| s.parse::<i64>().ok()),
            "collation": g(5)?,
        }));
    }
    Ok(Json(json!({ "database": db, "tables": tables })))
}

pub async fn table_structure(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;
    let cols = column_meta(&sess, &db, &table).await?;
    if cols.is_empty() {
        return Err(ApiError::not_found(format!("表 {} 不存在或无权访问", table)));
    }
    let pk = primary_key(&sess, &db, &table).await?;
    let columns: Vec<serde_json::Value> = cols
        .iter()
        .map(|c| {
            json!({
                "name": c.name,
                "dataType": c.data_type,
                "type": c.column_type,
                "nullable": c.nullable,
                "key": c.key,
                "extra": c.extra,
                "default": c.default,
                "collation": c.collation,
                "binary": crate::sqlutil::is_binary_type(&c.data_type),
            })
        })
        .collect();
    Ok(Json(json!({ "database": db, "table": table, "columns": columns, "primaryKey": pk })))
}
