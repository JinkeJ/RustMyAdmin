use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;

use crate::error::{ApiError, ApiResult};
use crate::session::AppState;
use crate::sqlutil::sql_string_lit;

/// Quote-escape the user@host literal (host defaults to % if empty)
fn user_host_lit(user: &str, host: &str) -> String {
    let host = if host.is_empty() { "%" } else { host };
    format!(
        "'{}'@'{}'",
        user.replace('\\', "\\\\").replace('\'', "\\'"),
        host.replace('\\', "\\\\").replace('\'', "\\'")
    )
}

// ============ User list ============

pub async fn list_users(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    let rows = sess
        .raw_rows("SELECT User, Host, plugin FROM mysql.user ORDER BY User, Host")
        .await
        .map_err(|e| {
            ApiError::forbidden(format!(
                "无法读取用户列表（当前账号可能无权访问 mysql.user）：{}",
                e.message
            ))
        })?;

    let mut users = Vec::with_capacity(rows.len());
    for r in rows {
        let g = |i: usize| -> ApiResult<Option<String>> { Ok(r.try_get_unchecked::<Option<String>, usize>(i)?) };
        users.push(json!({
            "user": g(0)?.unwrap_or_default(),
            "host": g(1)?.unwrap_or_default(),
            "plugin": g(2)?.unwrap_or_default(),
        }));
    }
    Ok(Json(json!({ "users": users })))
}

// ============ Create / delete / rename users ============

#[derive(Deserialize)]
pub struct UserHostReq {
    pub user: String,
    #[serde(default)]
    pub host: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateUserReq {
    pub user: String,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub password: String,
}

pub async fn create_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<CreateUserReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    if req.user.is_empty() {
        return Err(ApiError::bad("用户名不能为空"));
    }
    let uh = user_host_lit(&req.user, req.host.as_deref().unwrap_or("localhost"));
    let sql = format!(
        "CREATE USER {uh} IDENTIFIED BY {}",
        sql_string_lit(&req.password)
    );
    sess.raw_exec_ok(sql).await?;
    Ok(Json(json!({ "ok": true, "message": format!("用户 {uh} 已创建") })))
}

pub async fn drop_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<UserHostReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    let uh = user_host_lit(&req.user, req.host.as_deref().unwrap_or(""));
    let sql = format!("DROP USER {uh}");
    sess.raw_exec_ok(sql).await?;
    Ok(Json(json!({ "ok": true, "message": format!("用户 {uh} 已删除") })))
}

#[derive(Deserialize)]
pub struct RenameUserReq {
    pub user: String,
    #[serde(default)]
    pub host: Option<String>,
    pub to: String,
}

pub async fn rename_user(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<RenameUserReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    if req.to.is_empty() {
        return Err(ApiError::bad("新用户名不能为空"));
    }
    let from = user_host_lit(&req.user, req.host.as_deref().unwrap_or(""));
    let to = user_host_lit(&req.to, req.host.as_deref().unwrap_or(""));
    sess.raw_exec_ok(format!("RENAME USER {from} TO {to}"))
        .await?;
    Ok(Json(json!({ "ok": true, "message": format!("用户 {from} 已重命名为 {to}") })))
}

// ============ Change password ============

#[derive(Deserialize)]
pub struct ChangePasswordReq {
    pub user: String,
    #[serde(default)]
    pub host: Option<String>,
    pub password: String,
}

pub async fn change_password(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<ChangePasswordReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    if req.password.is_empty() {
        return Err(ApiError::bad("密码不能为空"));
    }
    let uh = user_host_lit(&req.user, req.host.as_deref().unwrap_or(""));
    // MySQL 5.7.6+ / MariaDB: ALTER USER
    let sql = format!(
        "ALTER USER {uh} IDENTIFIED BY {}",
        sql_string_lit(&req.password)
    );
    match sess.raw_exec_ok(sql).await {
        Ok(()) => Ok(Json(json!({ "ok": true, "message": format!("用户 {uh} 的密码已修改") }))),
        Err(e) => {
            // Legacy fallback: SET PASSWORD ... = PASSWORD('...') (removed in MySQL 8.0)
            if e.message.contains("syntax") || e.message.contains("1064") {
                let fallback = format!(
                    "SET PASSWORD FOR {uh} = PASSWORD({})",
                    sql_string_lit(&req.password)
                );
                sess.raw_exec_ok(fallback).await?;
                Ok(Json(json!({ "ok": true, "message": format!("用户 {uh} 的密码已修改") })))
            } else {
                Err(e)
            }
        }
    }
}

// ============ View grants / reload privileges ============

#[derive(Deserialize)]
pub struct GrantsQuery {
    pub user: String,
    pub host: Option<String>,
}

pub async fn show_grants(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Query(q): Query<GrantsQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    let uh = user_host_lit(&q.user, q.host.as_deref().unwrap_or(""));
    let rows = sess.raw_rows(format!("SHOW GRANTS FOR {uh}")).await?;
    let mut grants = Vec::with_capacity(rows.len());
    for r in rows {
        grants.push(r.try_get_unchecked::<String, usize>(0)?);
    }
    Ok(Json(json!({ "grants": grants })))
}

pub async fn flush_privileges(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    sess.raw_exec_ok("FLUSH PRIVILEGES").await?;
    Ok(Json(json!({ "ok": true, "message": "权限已重新加载（FLUSH PRIVILEGES）" })))
}

// ============ Visual grant editor (GRANT / REVOKE) ============

/// Static whitelist of supported privileges (lowercase matching, canonical uppercase output)
const USER_PRIVS: &[&str] = &[
    "select", "insert", "update", "delete", "file",
    "create", "alter", "drop", "index", "create view", "show view",
    "create temporary tables", "lock tables", "references", "trigger", "event",
    "create routine", "alter routine", "execute",
    "grant option", "show databases", "create user", "reload", "shutdown", "process",
    "replication client", "replication slave",
];

/// Normalize privilege list: whitelist validation, dedup, uppercase spelling; returns comma-joined string
fn norm_privs(list: &[String], what: &str) -> ApiResult<String> {
    let mut out: Vec<String> = Vec::new();
    for p in list {
        let lp = p.trim().to_ascii_lowercase();
        if lp.is_empty() {
            continue;
        }
        let canon = if lp == "all" || lp == "all privileges" {
            "ALL PRIVILEGES".to_string()
        } else {
            let found = USER_PRIVS
                .iter()
                .find(|x| **x == lp)
                .ok_or_else(|| ApiError::bad(format!("不支持的权限：{p}")))?;
            found.to_ascii_uppercase()
        };
        if !out.contains(&canon) {
            out.push(canon);
        }
    }
    if out.is_empty() {
        return Err(ApiError::bad(format!("请勾选要{what}的权限")));
    }
    Ok(out.join(", "))
}

#[derive(Deserialize)]
pub struct GrantsApplyReq {
    pub user: String,
    #[serde(default)]
    pub host: Option<String>,
    /// Privileges to grant
    #[serde(default)]
    pub grant: Vec<String>,
    /// Privileges to revoke
    #[serde(default)]
    pub revoke: Vec<String>,
    /// Scope: global / db / table
    pub scope: String,
    #[serde(default)]
    pub database: Option<String>,
    #[serde(default)]
    pub table: Option<String>,
}

pub async fn apply_grants(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<GrantsApplyReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    if req.user.trim().is_empty() {
        return Err(ApiError::bad("用户名不能为空"));
    }
    let uh = user_host_lit(req.user.trim(), req.host.as_deref().unwrap_or(""));
    // Pre-check: target user must already exist (otherwise MariaDB GRANT would implicitly create the account)
    {
        let host_lit = sql_string_lit(
            req.host
                .as_deref()
                .filter(|s| !s.is_empty())
                .unwrap_or("%"),
        );
        let rows = sess
            .raw_rows(format!(
                "SELECT COUNT(*) FROM mysql.user WHERE User = {} AND Host = {}",
                sql_string_lit(req.user.trim()),
                host_lit
            ))
            .await
            .map_err(|e| {
                ApiError::forbidden(format!("无法访问 mysql.user：{}", e.message))
            })?;
        let exists: i64 = rows
            .first()
            .and_then(|r| r.try_get_unchecked::<String, usize>(0).ok())
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        if exists == 0 {
            return Err(ApiError::not_found(format!(
                "用户 {} 不存在，请先创建用户",
                uh
            )));
        }
    }
    // Map scope to target object
    let target = match req.scope.as_str() {
        "global" => "*.*".to_string(),
        "db" => {
            let db = req
                .database
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| ApiError::bad("请选择数据库"))?;
            crate::sqlutil::check_ident(db, "数据库")?;
            format!("{}.*", crate::sqlutil::ident(db))
        }
        "table" => {
            let db = req
                .database
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| ApiError::bad("请选择数据库"))?;
            let tbl = req
                .table
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| ApiError::bad("请填写表名"))?;
            crate::sqlutil::check_ident(db, "数据库")?;
            crate::sqlutil::check_ident(tbl, "数据表")?;
            format!("{}.{}", crate::sqlutil::ident(db), crate::sqlutil::ident(tbl))
        }
        _ => return Err(ApiError::bad("作用域无效")),
    };
    if req.grant.is_empty() && req.revoke.is_empty() {
        return Err(ApiError::bad("请先勾选权限"));
    }
    if !req.grant.is_empty() {
        let privs = norm_privs(&req.grant, "授予")?;
        sess.raw_exec_ok(format!("GRANT {privs} ON {target} TO {uh}"))
            .await?;
    }
    if !req.revoke.is_empty() {
        let privs = norm_privs(&req.revoke, "收回")?;
        sess.raw_exec_ok(format!("REVOKE {privs} ON {target} FROM {uh}"))
            .await?;
    }
    Ok(Json(json!({ "ok": true, "message": "授权已更新" })))
}
