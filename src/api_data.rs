use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use sqlx::Row;

use crate::error::{ApiError, ApiResult};
use crate::exec::{binary_flags_of, column_meta, primary_key, ColumnMeta};
use crate::session::{AppState, Session};
use crate::sqlutil::{
    build_where, check_ident, ident, is_binary_type, is_hex, qualify, row_cells, sql_string_lit,
};

// ============ Browse data ============

#[derive(Deserialize)]
pub struct RowsQuery {
    pub page: Option<u64>,
    pub size: Option<u64>,
    pub order: Option<String>,
    pub dir: Option<String>,
    /// Quick search: LIKE match on all non-binary columns
    pub search: Option<String>,
}

pub async fn browse_rows(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
    Query(q): Query<RowsQuery>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;

    let cols = column_meta(&sess, &db, &table).await?;
    if cols.is_empty() {
        return Err(ApiError::not_found(format!("表 {} 不存在或无权访问", table)));
    }
    let pk = primary_key(&sess, &db, &table).await?;
    let flags = binary_flags_of(&cols);

    let qt = qualify(&db, &table);

    // Quick search: LIKE on non-binary columns (% _ \ escaped literally)
    let mut where_clause = String::new();
    let search_term = q.search.as_deref().map(str::trim).filter(|s| !s.is_empty());
    if let Some(s) = search_term {
        if s.chars().count() > 256 {
            return Err(ApiError::bad("搜索内容过长"));
        }
        let pat = sql_string_lit(&format!(
            "%{}%",
            s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
        ));
        let conds: Vec<String> = cols
            .iter()
            .filter(|c| !is_binary_type(&c.data_type))
            .map(|c| format!("{} LIKE {}", ident(&c.name), pat))
            .collect();
        if conds.is_empty() {
            return Err(ApiError::bad("此表没有可搜索的字段"));
        }
        where_clause = format!(" WHERE {}", conds.join(" OR "));
    }

    // total row count
    let rows = sess
        .raw_rows(format!("SELECT COUNT(*) FROM {qt}{where_clause}"))
        .await?;
    let total: i64 = rows
        .first()
        .and_then(|r| {
            r.try_get_unchecked::<String, usize>(0)
                .ok()
                .and_then(|s| s.parse().ok())
        })
        .unwrap_or(0);

    // pagination
    let size = q.size.unwrap_or(25).clamp(1, 500);
    let pages = ((total as u64).saturating_sub(1) / size) + 1;
    let page = q.page.unwrap_or(1).clamp(1, pages);

    // sorting (column-name whitelist check)
    let mut order_clause = String::new();
    if let Some(col) = &q.order {
        if let Some(m) = cols.iter().find(|c| &c.name == col) {
            let dir = if q.dir.as_deref() == Some("desc") { "DESC" } else { "ASC" };
            order_clause = format!(" ORDER BY {} {}", ident(&m.name), dir);
        }
    }

    let offset = (page - 1) * size;
    let rows = sess
        .raw_rows(format!(
            "SELECT * FROM {qt}{where_clause}{order_clause} LIMIT {size} OFFSET {offset}"
        ))
        .await?;

    // key columns: use all columns for row location when there is no primary key
    let key_cols: Vec<String> = if pk.is_empty() {
        cols.iter().map(|c| c.name.clone()).collect()
    } else {
        pk.clone()
    };
    let key_idx: Vec<usize> = key_cols
        .iter()
        .map(|k| cols.iter().position(|c| &c.name == k).unwrap_or(0))
        .collect();

    let mut out_rows = Vec::with_capacity(rows.len());
    for r in &rows {
        let cells = row_cells(r, Some(&flags));
        let key: Vec<serde_json::Value> = key_idx.iter().map(|&i| cells[i].desc()).collect();
        out_rows.push(json!({
            "k": key,
            "c": cells.iter().map(|c| c.display()).collect::<Vec<_>>(),
        }));
    }

    let columns: Vec<serde_json::Value> = cols
        .iter()
        .map(|c| {
            json!({
                "name": c.name,
                "type": c.column_type,
                "dataType": c.data_type,
                "nullable": c.nullable,
                "key": c.key,
                "extra": c.extra,
                "default": c.default,
                "binary": is_binary_type(&c.data_type),
            })
        })
        .collect();

    Ok(Json(json!({
        "database": db,
        "table": table,
        "columns": columns,
        "primaryKey": pk,
        "keyColumns": key_cols,
        "rows": out_rows,
        "total": total,
        "page": page,
        "size": size,
        "pages": pages,
        "order": q.order,
        "dir": q.dir,
        "search": search_term,
    })))
}

// ============ Row location helpers ============

async fn key_cols_of(sess: &Arc<Session>, db: &str, table: &str) -> ApiResult<Vec<String>> {
    let pk = primary_key(sess, db, table).await?;
    if !pk.is_empty() {
        return Ok(pk);
    }
    let cols = column_meta(sess, db, table).await?;
    Ok(cols.into_iter().map(|c| c.name).collect())
}

// ============ Delete rows (single/multiple, in a transaction) ============

#[derive(Deserialize)]
pub struct DeleteRowsReq {
    pub keys: Vec<Vec<serde_json::Value>>,
}

pub async fn delete_rows(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
    Json(req): Json<DeleteRowsReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;
    if req.keys.is_empty() {
        return Err(ApiError::bad("未选择要删除的行"));
    }
    let kcols = key_cols_of(&sess, &db, &table).await?;
    let qt = qualify(&db, &table);

    let mut stmts = Vec::with_capacity(req.keys.len());
    for key in &req.keys {
        let cond = build_where(&kcols, key)?;
        stmts.push(format!("DELETE FROM {qt} WHERE {cond}"));
    }

    let affected = sess.exec_tx(stmts).await?;
    Ok(Json(json!({ "ok": true, "affectedRows": affected, "message": format!("已删除 {affected} 行") })))
}

// ============ Edit single row ============

#[derive(Deserialize)]
pub struct RowChange {
    pub col: String,
    pub value: String,
    #[serde(default)]
    pub is_null: bool,
    /// Value is a function expression (e.g. CURRENT_TIMESTAMP); whitelist entries only
    #[serde(default)]
    pub is_expr: bool,
}

#[derive(Deserialize)]
pub struct UpdateRowReq {
    pub key: Vec<serde_json::Value>,
    pub changes: Vec<RowChange>,
}

/// Convert user input into a SQL value expression: NULL / function expression (whitelist) / hex (binary columns) / escaped text
fn input_value_sql(data_type: &str, change: &RowChange) -> ApiResult<String> {
    if change.is_null {
        return Ok("NULL".to_string());
    }
    if change.is_expr {
        let u = change.value.trim().to_ascii_uppercase();
        let ok = match u.as_str() {
            "CURRENT_TIMESTAMP" => {
                matches!(data_type.to_ascii_lowercase().as_str(), "datetime" | "timestamp")
            }
            "CURRENT_DATE" => data_type.eq_ignore_ascii_case("date"),
            "CURRENT_TIME" => data_type.eq_ignore_ascii_case("time"),
            _ => false,
        };
        if !ok {
            return Err(ApiError::bad(format!("字段类型 {data_type} 不支持表达式 {u}")));
        }
        return Ok(u);
    }
    if is_binary_type(data_type) {
        let hex = change.value.trim().trim_start_matches("0x").to_string();
        if is_hex(&hex) {
            Ok(format!("UNHEX('{}')", hex))
        } else {
            Err(ApiError::bad(
                "二进制列请输入十六进制（可带 0x 前缀），例如 4D5A 或 0x4D5A",
            ))
        }
    } else {
        Ok(sql_string_lit(&change.value))
    }
}

pub async fn update_row(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
    Json(req): Json<UpdateRowReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;
    if req.changes.is_empty() {
        return Err(ApiError::bad("没有任何修改"));
    }
    let kcols = key_cols_of(&sess, &db, &table).await?;
    let cols = column_meta(&sess, &db, &table).await?;

    let mut sets = Vec::with_capacity(req.changes.len());
    for ch in &req.changes {
        let meta = cols
            .iter()
            .find(|c| c.name == ch.col)
            .ok_or_else(|| ApiError::bad(format!("字段 {} 不存在", ch.col)))?;
        sets.push(format!("{} = {}", ident(&ch.col), input_value_sql(&meta.data_type, ch)?));
    }
    let cond = build_where(&kcols, &req.key)?;
    let sql = format!("UPDATE {} SET {} WHERE {} LIMIT 1", qualify(&db, &table), sets.join(", "), cond);

    let (affected, _) = sess.raw_exec(sql).await?;
    Ok(Json(json!({ "ok": true, "affectedRows": affected, "message": format!("已更新 {affected} 行") })))
}

// ============ Insert row ============

#[derive(Deserialize)]
pub struct InsertRowReq {
    pub values: Vec<RowChange>,
}

pub async fn insert_row(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
    Json(req): Json<InsertRowReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;
    let cols = column_meta(&sess, &db, &table).await?;
    if cols.is_empty() {
        return Err(ApiError::not_found("表不存在"));
    }

    let mut names = Vec::new();
    let mut vals = Vec::new();
    for ch in &req.values {
        let meta = cols
            .iter()
            .find(|c| c.name == ch.col)
            .ok_or_else(|| ApiError::bad(format!("字段 {} 不存在", ch.col)))?;
        // Auto-increment/auto-generated fields with an empty value and NULL unchecked: skip
        if !ch.is_null && ch.value.is_empty() && !meta.extra.is_empty() {
            continue;
        }
        names.push(ident(&ch.col));
        vals.push(input_value_sql(&meta.data_type, ch)?);
    }
    if names.is_empty() {
        return Err(ApiError::bad("没有任何要插入的字段"));
    }
    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        qualify(&db, &table),
        names.join(", "),
        vals.join(", ")
    );
    let (affected, insert_id) = sess.raw_exec(sql).await?;
    Ok(Json(json!({ "ok": true, "affectedRows": affected, "insertId": insert_id, "message": "已插入 1 行" })))
}

// ============ Truncate / drop / rename table ============

pub async fn truncate_table(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;
    sess.raw_exec_ok(format!("TRUNCATE TABLE {}", qualify(&db, &table)))
        .await?;
    Ok(Json(json!({ "ok": true, "message": format!("表 {} 已清空", table) })))
}

pub async fn drop_table(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;
    sess.raw_exec_ok(format!("DROP TABLE IF EXISTS {}", qualify(&db, &table)))
        .await?;
    Ok(Json(json!({ "ok": true, "message": format!("表 {} 已删除", table) })))
}

#[derive(Deserialize)]
pub struct RenameReq {
    pub to: String,
}

pub async fn rename_table(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
    Json(req): Json<RenameReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "原数据表")?;
    check_ident(&req.to, "新数据表")?;
    if table == req.to {
        return Err(ApiError::bad("新名称与原名称相同"));
    }
    sess.raw_exec_ok(format!(
        "RENAME TABLE {} TO {}",
        qualify(&db, &table),
        qualify(&db, &req.to)
    ))
    .await?;
    Ok(Json(json!({ "ok": true, "message": format!("表已重命名为 {}", req.to) })))
}

// ============ Columns: rename / drop / add / move ============

/// Rebuild the full column definition from information_schema so that type/charset/default/auto-increment are not lost
fn rebuild_column_def(meta: &ColumnMeta) -> String {
    let mut def = meta.column_type.clone();
    // Charset/collation must immediately follow the type (column definition syntax order)
    if let Some(cs) = &meta.collation {
        if let Some(charset) = cs.split('_').next() {
            def.push_str(&format!(" CHARACTER SET {}", ident(charset)));
            def.push_str(&format!(" COLLATE {}", ident(cs)));
        }
    }
    if !meta.nullable {
        def.push_str(" NOT NULL");
    } else {
        def.push_str(" NULL");
    }
    // MariaDB stores COLUMN_DEFAULT of "nullable with no explicit default" columns as the string "NULL";
    // it must be treated as implicit NULL, otherwise an invalid DEFAULT 'NULL' would be generated
    let d = meta.default.as_deref().filter(|d| *d != "NULL");
    if let Some(d) = d {
        let is_expr = d.starts_with("CURRENT_TIMESTAMP")
            || d.starts_with("CURRENT_DATE")
            || d.starts_with("CURRENT_TIME")
            || d.starts_with("CURRENT_USER")
            || meta.extra.contains("DEFAULT_GENERATED");
        if is_expr {
            def.push_str(&format!(" DEFAULT {}", d));
        } else {
            def.push_str(&format!(" DEFAULT {}", sql_string_lit(d)));
        }
    } else if meta.nullable {
        def.push_str(" DEFAULT NULL");
    }
    if meta.extra.contains("auto_increment") {
        def.push_str(" AUTO_INCREMENT");
    }
    if let Some(pos) = meta.extra.find("on update") {
        let tail = meta.extra[pos..].trim();
        if !tail.is_empty() {
            def.push_str(&format!(" {}", tail));
        }
    }
    def
}

pub async fn rename_column(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table, col)): Path<(String, String, String)>,
    Json(req): Json<RenameReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;
    check_ident(&col, "字段")?;
    check_ident(&req.to, "新字段")?;

    let cols = column_meta(&sess, &db, &table).await?;
    let meta = cols
        .iter()
        .find(|c| c.name == col)
        .ok_or_else(|| ApiError::bad(format!("字段 {col} 不存在")))?;
    if meta.extra.contains("VIRTUAL") || meta.extra.contains("STORED") {
        return Err(ApiError::bad("暂不支持重命名生成列（GENERATED ALWAYS AS）"));
    }

    let def = rebuild_column_def(meta);
    let sql = format!(
        "ALTER TABLE {} CHANGE {} {} {}",
        qualify(&db, &table),
        ident(&col),
        ident(&req.to),
        def
    );
    sess.raw_exec_ok(sql).await?;
    Ok(Json(json!({ "ok": true, "message": format!("字段已重命名为 {}", req.to) })))
}

#[derive(Deserialize)]
pub struct MoveColumnReq {
    /// "first" (move to the front) or a target column name (place after that column)
    pub position: String,
}

/// Reorder a column (MODIFY COLUMN ... FIRST / AFTER); the column definition is kept unchanged
pub async fn move_column(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table, col)): Path<(String, String, String)>,
    Json(req): Json<MoveColumnReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;
    check_ident(&col, "字段")?;
    let cols = column_meta(&sess, &db, &table).await?;
    let meta = cols
        .iter()
        .find(|c| c.name == col)
        .ok_or_else(|| ApiError::bad(format!("字段 {col} 不存在")))?;
    if meta.extra.contains("VIRTUAL") || meta.extra.contains("STORED") {
        return Err(ApiError::bad("暂不支持移动生成列（GENERATED ALWAYS AS）"));
    }
    let pos_clause = if req.position.eq_ignore_ascii_case("first") {
        "FIRST".to_string()
    } else {
        check_ident(&req.position, "目标字段")?;
        if req.position == col {
            return Err(ApiError::bad("不能移动到自己后面"));
        }
        if !cols.iter().any(|c| c.name == req.position) {
            return Err(ApiError::bad(format!("字段 {} 不存在", req.position)));
        }
        format!("AFTER {}", ident(&req.position))
    };
    let def = rebuild_column_def(meta);
    let sql = format!(
        "ALTER TABLE {} MODIFY COLUMN {} {} {}",
        qualify(&db, &table),
        ident(&col),
        def,
        pos_clause
    );
    sess.raw_exec_ok(sql).await?;
    Ok(Json(json!({ "ok": true, "message": format!("字段 {col} 位置已调整") })))
}

#[derive(Deserialize)]
pub struct AddColumnReq {
    pub name: String,
    pub col_type: String,
    #[serde(default)]
    pub length: String,
    #[serde(default)]
    pub nullable: bool,
    #[serde(default)]
    pub unsigned: bool,
    #[serde(default)]
    pub auto_increment: bool,
    #[serde(default)]
    pub default: Option<String>,
}

pub async fn add_column(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
    Json(req): Json<AddColumnReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;
    check_ident(&req.name, "字段")?;
    // Auto-increment pre-checks: type must be in the integer family, must be NOT NULL, have no default, and the table must not already have an auto-increment column
    if req.auto_increment {
        let cols = column_meta(&sess, &db, &table).await?;
        if let Some(other) = cols
            .iter()
            .find(|c| c.extra.contains("auto_increment"))
        {
            return Err(ApiError::bad(format!("表中已有自增字段 {}", other.name)));
        }
    }
    // Parse the type (legacy "VARCHAR(50)" / "INT UNSIGNED" still accepted; the new frontend sends only the base type + length/unsigned fields)
    let (ty, frag, unsigned) = parse_col_type(&req.col_type, &req.length, req.unsigned)?;
    if req.auto_increment {
        if !AI_TYPES.contains(&ty.as_str()) {
            return Err(ApiError::bad(format!("{ty} 类型不支持 AUTO_INCREMENT")));
        }
        if req.nullable {
            return Err(ApiError::bad("自增字段必须为 NOT NULL"));
        }
        if req.default.as_deref().map(str::trim).filter(|s| !s.is_empty()).is_some() {
            return Err(ApiError::bad("自增字段不能设置默认值"));
        }
    }
    let mut def = format!("{} {}{}", ident(&req.name), ty, frag);
    if unsigned {
        def.push_str(" UNSIGNED");
    }
    if req.nullable {
        def.push_str(" NULL");
    } else {
        def.push_str(" NOT NULL");
    }
    if let Some(d) = req.default.as_deref().map(str::trim).filter(|d| !d.is_empty()) {
        def.push_str(&default_clause(&ty, req.nullable, d)?);
    }
    // Auto-create an index for auto-increment columns (MySQL requires them to be a key)
    let sql = if req.auto_increment {
        def.push_str(" AUTO_INCREMENT");
        format!(
            "ALTER TABLE {} ADD COLUMN {}, ADD KEY ({})",
            qualify(&db, &table),
            def,
            ident(&req.name)
        )
    } else {
        format!("ALTER TABLE {} ADD COLUMN {}", qualify(&db, &table), def)
    };
    sess.raw_exec_ok(sql).await?;
    Ok(Json(json!({ "ok": true, "message": format!("字段 {} 已添加", req.name) })))
}

pub async fn drop_column(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table, col)): Path<(String, String, String)>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;
    check_ident(&col, "字段")?;
    let sql = format!("ALTER TABLE {} DROP COLUMN {}", qualify(&db, &table), ident(&col));
    sess.raw_exec_ok(sql).await?;
    Ok(Json(json!({ "ok": true, "message": format!("字段 {} 已删除", col) })))
}

#[derive(Deserialize)]
pub struct ModifyColumnReq {
    /// New column name (same as the original: modify attributes only; different: also rename)
    pub name: String,
    #[serde(rename = "type")]
    pub col_type: String,
    #[serde(default)]
    pub length: String,
    #[serde(default)]
    pub nullable: bool,
    #[serde(default)]
    pub unsigned: bool,
    #[serde(default)]
    pub auto_increment: bool,
    #[serde(default)]
    pub default: Option<String>,
}

/// Modify an existing column (rename + type/length/nullability/unsigned/default/auto-increment in one step)
pub async fn modify_column(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table, col)): Path<(String, String, String)>,
    Json(req): Json<ModifyColumnReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;
    check_ident(&col, "字段")?;
    check_ident(&req.name, "新字段名")?;

    let cols = column_meta(&sess, &db, &table).await?;
    if !cols.iter().any(|c| c.name == col) {
        return Err(ApiError::not_found(format!("字段 {col} 不存在")));
    }
    if req.auto_increment {
        // the table already has another auto-increment column
        if let Some(other) = cols
            .iter()
            .find(|c| c.name != col && c.extra.contains("auto_increment"))
        {
            return Err(ApiError::bad(format!("表中已有自增字段 {}", other.name)));
        }
        // auto-increment column must be the primary key
        let pk = primary_key(&sess, &db, &table).await?;
        if !pk.iter().any(|p| p == &col) {
            return Err(ApiError::bad("自增字段必须是主键，当前字段不是主键"));
        }
    }

    // Reuse the create-table single-column definition builder (primary_key is only used for the AI↔PK consistency check)
    let def = build_column_def(&NewColumnDef {
        name: req.name.clone(),
        col_type: req.col_type.clone(),
        length: req.length.clone(),
        nullable: req.nullable,
        unsigned: req.unsigned,
        auto_increment: req.auto_increment,
        primary_key: req.auto_increment,
        default: req.default.clone(),
    })?;

    let sql = format!(
        "ALTER TABLE {} CHANGE COLUMN {} {}",
        qualify(&db, &table),
        ident(&col),
        def
    );
    sess.raw_exec_ok(sql).await?;
    let msg = if req.name == col {
        format!("字段 {col} 已修改")
    } else {
        format!("字段 {col} 已修改为 {}", req.name)
    };
    Ok(Json(json!({ "ok": true, "message": msg })))
}

// ============ Type whitelist and table creation ============

/// Allowed column types (matches the frontend dropdown)
const COL_TYPES: &[&str] = &[
    "TINYINT", "SMALLINT", "MEDIUMINT", "INT", "BIGINT", "DECIMAL", "FLOAT", "DOUBLE", "BIT",
    "DATE", "TIME", "YEAR", "DATETIME", "TIMESTAMP",
    "CHAR", "VARCHAR", "TINYTEXT", "TEXT", "MEDIUMTEXT", "LONGTEXT", "ENUM", "SET",
    "BINARY", "VARBINARY", "TINYBLOB", "BLOB", "MEDIUMBLOB", "LONGBLOB", "JSON",
];
/// Types that accept UNSIGNED
const UNSIGNED_TYPES: &[&str] = &[
    "TINYINT", "SMALLINT", "MEDIUMINT", "INT", "BIGINT", "DECIMAL", "FLOAT", "DOUBLE",
];
/// Types that accept AUTO_INCREMENT
const AI_TYPES: &[&str] = &["TINYINT", "SMALLINT", "MEDIUMINT", "INT", "BIGINT"];
/// Types that require a length
const NEED_LEN_TYPES: &[&str] = &["VARCHAR", "CHAR", "VARBINARY", "BINARY"];
/// Allowed storage engines
const ENGINES: &[&str] = &["InnoDB", "MyISAM", "MEMORY", "Aria", "CSV"];

/// Numeric length string: n or p,s (e.g. 100 / 8,2)
fn is_len_digits(s: &str) -> bool {
    !s.is_empty() && s.len() <= 12 && {
        let mut comma = false;
        s.chars().all(|c| match c {
            '0'..='9' => true,
            ',' if !comma => {
                comma = true;
                true
            }
            _ => false,
        })
    }
}

/// Parse and validate the type string:
/// - New format: col_type carries the base type ("VARCHAR"); length comes from the length field
/// - Legacy format: col_type embeds "VARCHAR(50)" / "INT UNSIGNED" (length is ignored)
/// Returns (type, parenthesized fragment, whether UNSIGNED); col_type is case-insensitive
fn parse_col_type(col_type: &str, length: &str, unsigned_req: bool) -> ApiResult<(String, String, bool)> {
    let mut s = col_type.trim();
    if s.is_empty() {
        return Err(ApiError::bad("字段类型不能为空"));
    }
    // Strip a possible UNSIGNED suffix (only when it appears as a standalone word)
    let mut unsigned = unsigned_req;
    let lower = s.to_ascii_lowercase();
    if lower.ends_with("unsigned") && !lower.ends_with(" unsigned") {
        let head = &s[..s.len() - "unsigned".len()];
        if head.chars().last().map_or(false, |c| !c.is_ascii_alphabetic()) {
            s = head.trim_end();
            unsigned = true;
        }
    } else if lower.ends_with(" unsigned") {
        s = &s[..s.len() - " unsigned".len()];
        unsigned = true;
    }
    // Strip the optional parentheses
    let (base, inner) = match s.find('(') {
        Some(i) => {
            if !s.ends_with(')') {
                return Err(ApiError::bad("字段类型格式无效"));
            }
            (s[..i].trim(), Some(s[i + 1..s.len() - 1].to_string()))
        }
        None => (s.trim(), None),
    };
    let ty = base.to_ascii_uppercase();
    if !COL_TYPES.contains(&ty.as_str()) {
        return Err(ApiError::bad(format!("不支持的字段类型：{base}")));
    }
    if unsigned && !UNSIGNED_TYPES.contains(&ty.as_str()) {
        return Err(ApiError::bad(format!("{ty} 类型不支持 UNSIGNED")));
    }
    let len_str = inner.unwrap_or_else(|| length.trim().to_string());
    let frag = length_frag(&ty, &len_str)?;
    Ok((ty, frag, unsigned))
}

/// Build the length/value-list parenthesized fragment:
/// - ENUM/SET: value list (comma-separated; unquoted values are auto-quoted)
/// - Others: numeric length; mandatory for NEED_LEN_TYPES
/// Returns an empty string when no parentheses should be added
fn length_frag(ty: &str, len: &str) -> ApiResult<String> {
    let len = len.trim();
    if ty == "ENUM" || ty == "SET" {
        if len.is_empty() {
            return Err(ApiError::bad("ENUM/SET 需要提供取值列表，如 'a','b'"));
        }
        if len.len() > 512 {
            return Err(ApiError::bad("取值列表过长"));
        }
        let mut items = Vec::new();
        for part in len.split(',') {
            let p = part.trim();
            if p.is_empty() {
                return Err(ApiError::bad("取值列表格式无效"));
            }
            let inner = if p.len() >= 2 && p.starts_with('\'') && p.ends_with('\'') {
                &p[1..p.len() - 1]
            } else {
                p
            };
            items.push(sql_string_lit(inner));
        }
        return Ok(format!("({})", items.join(",")));
    }
    if len.is_empty() {
        if NEED_LEN_TYPES.contains(&ty) {
            return Err(ApiError::bad(format!("{ty} 类型必须填写长度")));
        }
        return Ok(String::new());
    }
    if !is_len_digits(len) {
        return Err(ApiError::bad("长度格式无效（应为数字，如 100 或 8,2）"));
    }
    Ok(format!("({len})"))
}

/// Build the default-value clause (returns e.g. " DEFAULT 'x'")
fn default_clause(ty: &str, nullable: bool, d: &str) -> ApiResult<String> {
    if d.eq_ignore_ascii_case("null") {
        if !nullable {
            return Err(ApiError::bad("NOT NULL 字段不能用 NULL 作默认值"));
        }
        return Ok(" DEFAULT NULL".into());
    }
    let du = d.to_ascii_uppercase();
    // CURRENT_TIMESTAMP[(n)] only for DATETIME/TIMESTAMP; CURRENT_DATE only for DATE
    let ts_ok = (ty == "DATETIME" || ty == "TIMESTAMP")
        && (du == "CURRENT_TIMESTAMP"
            || (du.starts_with("CURRENT_TIMESTAMP(")
                && du.ends_with(')')
                && du[18..du.len() - 1]
                    .chars()
                    .all(|c| c.is_ascii_digit())
                && !du[18..du.len() - 1].is_empty()));
    let date_ok = ty == "DATE" && du == "CURRENT_DATE";
    if ts_ok || date_ok {
        return Ok(format!(" DEFAULT {du}"));
    }
    if du.starts_with("CURRENT_") || du == "NOW()" {
        return Err(ApiError::bad(
            "函数默认值仅支持：DATE→CURRENT_DATE，DATETIME/TIMESTAMP→CURRENT_TIMESTAMP",
        ));
    }
    Ok(format!(" DEFAULT {}", sql_string_lit(d)))
}

#[derive(Deserialize)]
pub struct NewColumnDef {
    pub name: String,
    #[serde(rename = "type")]
    pub col_type: String,
    #[serde(default)]
    pub length: String,
    #[serde(default)]
    pub nullable: bool,
    #[serde(default)]
    pub unsigned: bool,
    #[serde(default)]
    pub auto_increment: bool,
    #[serde(default)]
    pub primary_key: bool,
    #[serde(default)]
    pub default: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateTableReq {
    pub name: String,
    pub columns: Vec<NewColumnDef>,
    #[serde(default)]
    pub engine: Option<String>,
}

/// Single column definition → SQL fragment
fn build_column_def(c: &NewColumnDef) -> ApiResult<String> {
    check_ident(&c.name, "字段")?;
    let (ty, frag, unsigned) = parse_col_type(&c.col_type, &c.length, c.unsigned)?;
    let mut def = format!("{} {}{}", ident(&c.name), ty, frag);
    if unsigned {
        def.push_str(" UNSIGNED");
    }
    if c.nullable {
        def.push_str(" NULL");
    } else {
        def.push_str(" NOT NULL");
    }
    if let Some(d) = c.default.as_deref().map(str::trim).filter(|d| !d.is_empty()) {
        def.push_str(&default_clause(&ty, c.nullable, d)?);
    }
    if c.auto_increment {
        if !AI_TYPES.contains(&ty.as_str()) {
            return Err(ApiError::bad(format!("{ty} 类型不支持 AUTO_INCREMENT")));
        }
        if !c.primary_key {
            return Err(ApiError::bad("AUTO_INCREMENT 字段必须同时勾选主键"));
        }
        def.push_str(" AUTO_INCREMENT");
    }
    Ok(def)
}

pub async fn create_table(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(db): Path<String>,
    Json(req): Json<CreateTableReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&req.name, "数据表")?;
    if req.columns.is_empty() {
        return Err(ApiError::bad("至少需要一个字段"));
    }
    if req.columns.len() > 100 {
        return Err(ApiError::bad("字段数量过多（上限 100）"));
    }
    // duplicate-name check (case-insensitive)
    let mut seen = std::collections::HashSet::new();
    for c in &req.columns {
        if !seen.insert(c.name.to_ascii_lowercase()) {
            return Err(ApiError::bad(format!("字段名重复：{}", c.name)));
        }
    }
    // at most one AUTO_INCREMENT
    let ai_count = req.columns.iter().filter(|c| c.auto_increment).count();
    if ai_count > 1 {
        return Err(ApiError::bad("一张表只能有一个 AUTO_INCREMENT 字段"));
    }
    let mut defs = Vec::with_capacity(req.columns.len() + 1);
    for c in &req.columns {
        defs.push(build_column_def(c)?);
    }
    let pk: Vec<&NewColumnDef> = req.columns.iter().filter(|c| c.primary_key).collect();
    if !pk.is_empty() {
        defs.push(format!(
            "PRIMARY KEY ({})",
            pk.iter().map(|c| ident(&c.name)).collect::<Vec<_>>().join(", ")
        ));
    }
    // engine (whitelist, case-insensitive)
    let engine = req
        .engine
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("InnoDB");
    let engine_lc = engine.to_ascii_lowercase();
    if !ENGINES.iter().any(|e| e.to_ascii_lowercase() == engine_lc) {
        return Err(ApiError::bad(format!("不支持的存储引擎：{engine}")));
    }
    // Hardcode utf8mb4_general_ci (users no longer choose a collation)
    let sql = format!(
        "CREATE TABLE {} (\n  {}\n) ENGINE={} DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci",
        qualify(&db, &req.name),
        defs.join(",\n  "),
        engine_lc
    );
    sess.raw_exec_ok(sql).await?;
    Ok(Json(json!({ "ok": true, "message": format!("表 {} 已创建", req.name), "table": req.name })))
}

// ============ Indexes ============

/// List all indexes of a table (information_schema.STATISTICS grouped by index name)
pub async fn list_indexes(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;
    let sql = format!(
        "SELECT INDEX_NAME, NON_UNIQUE, SEQ_IN_INDEX, COLUMN_NAME, SUB_PART, INDEX_TYPE \
         FROM information_schema.STATISTICS \
         WHERE TABLE_SCHEMA = {} AND TABLE_NAME = {} \
         ORDER BY INDEX_NAME, SEQ_IN_INDEX",
        sql_string_lit(&db),
        sql_string_lit(&table)
    );
    let rows = sess.raw_rows(sql).await?;
    let mut out: Vec<serde_json::Value> = Vec::new();
    let mut pos: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for r in rows {
        let get = |i: usize| -> ApiResult<Option<String>> { Ok(r.try_get_unchecked::<Option<String>, usize>(i)?) };
        let name = get(0)?.unwrap_or_default();
        if let Some(&i) = pos.get(&name) {
            if let Some(cols) = out[i]["columns"].as_array_mut() {
                cols.push(json!({
                    "name": get(3)?.unwrap_or_default(),
                    "subPart": get(4)?,
                }));
            }
            continue;
        }
        pos.insert(name.clone(), out.len());
        out.push(json!({
            "name": name,
            "unique": get(1)?.as_deref() != Some("1"),
            "isPrimary": name == "PRIMARY",
            "indexType": get(5)?.unwrap_or_default(),
            "columns": [ {
                "name": get(3)?.unwrap_or_default(),
                "subPart": get(4)?,
            } ],
        }));
    }
    Ok(Json(json!({ "indexes": out })))
}

#[derive(Deserialize)]
pub struct IndexColumnReq {
    pub name: String,
    #[serde(default)]
    pub length: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateIndexReq {
    #[serde(default)]
    pub name: Option<String>,
    pub kind: String,
    pub columns: Vec<IndexColumnReq>,
}

/// Create an index: kind ∈ primary / unique / index / fulltext / spatial
pub async fn create_index(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
    Json(req): Json<CreateIndexReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;
    let kind = req.kind.trim().to_ascii_lowercase();
    if !matches!(kind.as_str(), "primary" | "unique" | "index" | "fulltext" | "spatial") {
        return Err(ApiError::bad("索引类型无效"));
    }
    if req.columns.is_empty() {
        return Err(ApiError::bad("至少选择一个字段"));
    }
    if req.columns.len() > 16 {
        return Err(ApiError::bad("索引字段过多（上限 16）"));
    }
    // Column existence + duplicates + prefix length
    let cols = column_meta(&sess, &db, &table).await?;
    let mut seen = std::collections::HashSet::new();
    let mut col_sql = Vec::with_capacity(req.columns.len());
    for c in &req.columns {
        check_ident(&c.name, "索引字段")?;
        if !cols.iter().any(|m| m.name.eq_ignore_ascii_case(&c.name)) {
            return Err(ApiError::bad(format!("字段 {} 不存在", c.name)));
        }
        if !seen.insert(c.name.to_ascii_lowercase()) {
            return Err(ApiError::bad(format!("索引字段重复：{}", c.name)));
        }
        let mut part = ident(&c.name);
        if let Some(l) = c.length.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            if l.len() > 4 || l == "0" || !l.chars().all(|c| c.is_ascii_digit()) {
                return Err(ApiError::bad("索引前缀长度无效"));
            }
            part.push_str(&format!("({l})"));
        }
        col_sql.push(part);
    }
    let cols_joined = col_sql.join(", ");
    // Index name (ignored for primary; auto-generated when empty)
    let name = if kind == "primary" {
        None
    } else {
        match req.name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            Some(n) => {
                check_ident(n, "索引名")?;
                Some(n.to_string())
            }
            None => {
                let auto = format!(
                    "idx_{}",
                    req.columns
                        .iter()
                        .map(|c| c.name.as_str())
                        .collect::<Vec<_>>()
                        .join("_")
                );
                Some(auto.chars().take(64).collect())
            }
        }
    };
    let key_sql = match kind.as_str() {
        "primary" => format!("ADD PRIMARY KEY ({cols_joined})"),
        "unique" => format!("ADD UNIQUE {} ({})", ident(name.as_deref().unwrap_or_default()), cols_joined),
        "fulltext" => format!("ADD FULLTEXT {} ({})", ident(name.as_deref().unwrap_or_default()), cols_joined),
        "spatial" => format!("ADD SPATIAL {} ({})", ident(name.as_deref().unwrap_or_default()), cols_joined),
        _ => format!("ADD INDEX {} ({})", ident(name.as_deref().unwrap_or_default()), cols_joined),
    };
    let sql = format!("ALTER TABLE {} {key_sql}", qualify(&db, &table));
    sess.raw_exec_ok(sql).await?;
    Ok(Json(json!({ "ok": true, "message": format!("索引 {} 已创建", name.as_deref().unwrap_or("PRIMARY")) })))
}

/// Drop an index (look up the real index name on the server first, then choose DROP INDEX / DROP PRIMARY KEY)
pub async fn drop_index(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table, name)): Path<(String, String, String)>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;
    check_ident(&name, "索引")?;
    let rows = sess
        .raw_rows(format!(
            "SELECT DISTINCT INDEX_NAME FROM information_schema.STATISTICS \
             WHERE TABLE_SCHEMA = {} AND TABLE_NAME = {}",
            sql_string_lit(&db),
            sql_string_lit(&table)
        ))
        .await?;
    let real: Vec<String> = rows
        .iter()
        .map(|r| r.try_get_unchecked::<String, usize>(0).unwrap_or_default())
        .collect();
    let found = real
        .iter()
        .find(|n| n.eq_ignore_ascii_case(&name))
        .ok_or_else(|| ApiError::not_found(format!("索引 {name} 不存在")))?;
    let sql = if found == "PRIMARY" {
        format!("ALTER TABLE {} DROP PRIMARY KEY", qualify(&db, &table))
    } else {
        format!("ALTER TABLE {} DROP INDEX {}", qualify(&db, &table), ident(found))
    };
    sess.raw_exec_ok(sql).await?;
    Ok(Json(json!({ "ok": true, "message": format!("索引 {found} 已删除") })))
}

// ============ Foreign keys ============

/// ON DELETE / ON UPDATE rule mapping (frontend value → SQL keyword)
fn fk_rule(rule: Option<&str>) -> ApiResult<Option<&'static str>> {
    match rule.map(str::trim).filter(|s| !s.is_empty()) {
        None | Some("") => Ok(None),
        Some("cascade") => Ok(Some("CASCADE")),
        Some("restrict") => Ok(Some("RESTRICT")),
        Some("no_action") => Ok(Some("NO ACTION")),
        Some("set_null") => Ok(Some("SET NULL")),
        Some("set_default") => Ok(Some("SET DEFAULT")),
        Some(other) => Err(ApiError::bad(format!("外键规则无效：{other}"))),
    }
}

/// List a table's foreign keys (KEY_COLUMN_USAGE + REFERENTIAL_CONSTRAINTS merged)
pub async fn list_foreign_keys(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;
    let rows = sess
        .raw_rows(format!(
            "SELECT CONSTRAINT_NAME, COLUMN_NAME, ORDINAL_POSITION, REFERENCED_TABLE_SCHEMA, \
                    REFERENCED_TABLE_NAME, REFERENCED_COLUMN_NAME \
             FROM information_schema.KEY_COLUMN_USAGE \
             WHERE TABLE_SCHEMA = {} AND TABLE_NAME = {} AND REFERENCED_TABLE_NAME IS NOT NULL \
             ORDER BY CONSTRAINT_NAME, ORDINAL_POSITION",
            sql_string_lit(&db),
            sql_string_lit(&table)
        ))
        .await?;
    let mut out: Vec<serde_json::Value> = Vec::new();
    let mut pos: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for r in rows {
        let get = |i: usize| -> ApiResult<Option<String>> { Ok(r.try_get_unchecked::<Option<String>, usize>(i)?) };
        let name = get(0)?.unwrap_or_default();
        if let Some(&i) = pos.get(&name) {
            let colv = get(1)?.unwrap_or_default();
            let rcv = get(5)?.unwrap_or_default();
            let o = &mut out[i];
            if let Some(a) = o["columns"].as_array_mut() {
                a.push(json!(colv));
            }
            if let Some(a) = o["refColumns"].as_array_mut() {
                a.push(json!(rcv));
            }
            continue;
        }
        pos.insert(name.clone(), out.len());
        out.push(json!({
            "name": name,
            "columns": [ get(1)?.unwrap_or_default() ],
            "refDb": get(3)?.unwrap_or_default(),
            "refTable": get(4)?.unwrap_or_default(),
            "refColumns": [ get(5)?.unwrap_or_default() ],
            "onDelete": null,
            "onUpdate": null,
        }));
    }
    if !out.is_empty() {
        let rules = sess
            .raw_rows(format!(
                "SELECT CONSTRAINT_NAME, UPDATE_RULE, DELETE_RULE \
                 FROM information_schema.REFERENTIAL_CONSTRAINTS \
                 WHERE CONSTRAINT_SCHEMA = {} AND TABLE_NAME = {}",
                sql_string_lit(&db),
                sql_string_lit(&table)
            ))
            .await?;
        for r in rules {
            let name: String = r.try_get_unchecked::<String, usize>(0)?;
            let upd: Option<String> = r.try_get_unchecked::<Option<String>, usize>(1)?;
            let del: Option<String> = r.try_get_unchecked::<Option<String>, usize>(2)?;
            if let Some(&i) = pos.get(&name) {
                out[i]["onUpdate"] = json!(upd);
                out[i]["onDelete"] = json!(del);
            }
        }
    }
    Ok(Json(json!({ "foreignKeys": out })))
}

#[derive(Deserialize)]
pub struct CreateFkReq {
    #[serde(default)]
    pub name: Option<String>,
    pub columns: Vec<String>,
    #[serde(default)]
    pub ref_db: Option<String>,
    pub ref_table: String,
    pub ref_columns: Vec<String>,
    #[serde(default)]
    pub on_delete: Option<String>,
    #[serde(default)]
    pub on_update: Option<String>,
}

/// Create a foreign key
pub async fn create_foreign_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table)): Path<(String, String)>,
    Json(req): Json<CreateFkReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;
    check_ident(&req.ref_table, "引用表")?;
    let ref_db = req.ref_db.as_deref().map(str::trim).filter(|s| !s.is_empty()).unwrap_or(&db).to_string();
    check_ident(&ref_db, "引用数据库")?;
    if req.columns.is_empty() || req.ref_columns.is_empty() {
        return Err(ApiError::bad("请选择外键字段和引用字段"));
    }
    if req.columns.len() != req.ref_columns.len() {
        return Err(ApiError::bad("外键字段与引用字段数量必须一致"));
    }
    if req.columns.len() > 16 {
        return Err(ApiError::bad("外键字段过多（上限 16）"));
    }
    // Composite foreign-key columns must not repeat
    let mut seen_my = std::collections::HashSet::new();
    for c in &req.columns {
        if !seen_my.insert(c.to_ascii_lowercase()) {
            return Err(ApiError::bad(format!("外键字段重复：{c}")));
        }
    }
    let mut seen_ref = std::collections::HashSet::new();
    for c in &req.ref_columns {
        if !seen_ref.insert(c.to_ascii_lowercase()) {
            return Err(ApiError::bad(format!("引用字段重复：{c}")));
        }
    }
    // Local column existence check
    let cols = column_meta(&sess, &db, &table).await?;
    for c in &req.columns {
        check_ident(c, "外键字段")?;
        if !cols.iter().any(|m| m.name.eq_ignore_ascii_case(c)) {
            return Err(ApiError::bad(format!("字段 {} 不存在", c)));
        }
    }
    // Referenced-table column existence check
    let ref_cols = column_meta(&sess, &ref_db, &req.ref_table).await?;
    if ref_cols.is_empty() {
        return Err(ApiError::bad(format!("引用表 {}.{} 不存在或无权访问", ref_db, req.ref_table)));
    }
    for c in &req.ref_columns {
        check_ident(c, "引用字段")?;
        if !ref_cols.iter().any(|m| m.name.eq_ignore_ascii_case(c)) {
            return Err(ApiError::bad(format!("引用字段 {} 不存在", c)));
        }
    }
    // Constraint name (auto-generated when empty)
    let name = match req.name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(n) => {
            check_ident(n, "外键名")?;
            n.to_string()
        }
        None => format!("fk_{}_{}", table, req.columns.first().map(String::as_str).unwrap_or("t"))
            .chars()
            .take(64)
            .collect(),
    };
    let on_delete = fk_rule(req.on_delete.as_deref())?;
    let on_update = fk_rule(req.on_update.as_deref())?;
    let mut sql = format!(
        "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({})",
        qualify(&db, &table),
        ident(&name),
        req.columns.iter().map(|c| ident(c)).collect::<Vec<_>>().join(", "),
        qualify(&ref_db, &req.ref_table),
        req.ref_columns.iter().map(|c| ident(c)).collect::<Vec<_>>().join(", ")
    );
    if let Some(r) = on_delete {
        sql.push_str(&format!(" ON DELETE {r}"));
    }
    if let Some(r) = on_update {
        sql.push_str(&format!(" ON UPDATE {r}"));
    }
    sess.raw_exec_ok(sql).await?;
    Ok(Json(json!({ "ok": true, "message": format!("外键 {name} 已创建") })))
}

/// Drop a foreign key (look up the real constraint name on the server first)
pub async fn drop_foreign_key(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path((db, table, name)): Path<(String, String, String)>,
) -> ApiResult<Json<serde_json::Value>> {
    let sess = state.auth(&headers).await?;
    check_ident(&db, "数据库")?;
    check_ident(&table, "数据表")?;
    check_ident(&name, "外键")?;
    let rows = sess
        .raw_rows(format!(
            "SELECT DISTINCT CONSTRAINT_NAME FROM information_schema.REFERENTIAL_CONSTRAINTS \
             WHERE CONSTRAINT_SCHEMA = {} AND TABLE_NAME = {}",
            sql_string_lit(&db),
            sql_string_lit(&table)
        ))
        .await?;
    let real: Vec<String> = rows
        .iter()
        .map(|r| r.try_get_unchecked::<String, usize>(0).unwrap_or_default())
        .collect();
    let found = real
        .iter()
        .find(|n| n.eq_ignore_ascii_case(&name))
        .ok_or_else(|| ApiError::not_found(format!("外键 {name} 不存在")))?;
    let sql = format!(
        "ALTER TABLE {} DROP FOREIGN KEY {}",
        qualify(&db, &table),
        ident(found)
    );
    sess.raw_exec_ok(sql).await?;
    Ok(Json(json!({ "ok": true, "message": format!("外键 {found} 已删除") })))
}
