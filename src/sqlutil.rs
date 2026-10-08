use sqlx::mysql::MySqlRow;
use sqlx::{Column, Row};

use crate::error::{ApiError, ApiResult};

/// Normalized representation of cell values:
/// - S: text (numbers, dates, etc. as their string form; identical to the server's return under the text protocol)
/// - B: binary (stored as hex to guarantee lossless round-trips)
/// - N: NULL
#[derive(Debug, Clone)]
pub enum Cell {
    S(String),
    B(Vec<u8>),
    N,
}

pub fn hex_encode(b: &[u8]) -> String {
    let mut s = String::with_capacity(b.len() * 2);
    for byte in b {
        s.push_str(&format!("{byte:02X}"));
    }
    s
}

impl Cell {
    /// For frontend display: binaries are shown as 0x hex
    pub fn display(&self) -> Option<String> {
        match self {
            Cell::S(s) => Some(s.clone()),
            Cell::B(b) => Some(format!("0x{}", hex_encode(b))),
            Cell::N => None,
        }
    }

    /// Row primary key descriptor; sent back verbatim by the frontend to locate the row
    pub fn desc(&self) -> serde_json::Value {
        match self {
            Cell::S(s) => serde_json::json!({ "t": "s", "v": s }),
            Cell::B(b) => serde_json::json!({ "t": "b", "v": hex_encode(b) }),
            Cell::N => serde_json::json!({ "t": "n" }),
        }
    }
}

/// Decode a single column. All queries use the raw_sql text protocol:
/// - Text columns: decoded directly as UTF-8 strings (numbers/dates/DECIMAL are exact text)
/// - Binary columns (blob/binary/bit etc.): decoded from raw bytes into hex
/// - Non-UTF-8 bytes: fall back to binary handling
fn decode_cell(row: &MySqlRow, i: usize, force_binary: bool) -> Cell {
    if force_binary {
        match row.try_get_unchecked::<Option<Vec<u8>>, usize>(i) {
            Ok(Some(b)) => Cell::B(b),
            Ok(None) => Cell::N,
            Err(_) => match row.try_get_unchecked::<Option<String>, usize>(i) {
                Ok(Some(s)) => Cell::S(s),
                _ => Cell::N,
            },
        }
    } else {
        match row.try_get_unchecked::<Option<String>, usize>(i) {
            Ok(Some(s)) => Cell::S(s),
            Ok(None) => Cell::N,
            Err(_) => match row.try_get_unchecked::<Option<Vec<u8>>, usize>(i) {
                Ok(Some(b)) => Cell::B(b),
                _ => Cell::N,
            },
        }
    }
}

/// Get all cells of a row. binary_flags maps one-to-one to columns (None = treat all as text)
pub fn row_cells(row: &MySqlRow, binary_flags: Option<&[bool]>) -> Vec<Cell> {
    let n = row.columns().len();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let force = binary_flags.map_or(false, |f| f.get(i).copied().unwrap_or(false));
        out.push(decode_cell(row, i, force));
    }
    out
}

pub fn row_columns(row: &MySqlRow) -> Vec<String> {
    row.columns().iter().map(|c| c.name().to_string()).collect()
}

/// Single-quoted string literal; handles backslashes and common control characters
pub fn sql_string_lit(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        match c {
            '\'' => out.push_str("\\'"),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\0' => out.push_str("\\0"),
            '\x1a' => out.push_str("\\Z"),
            _ => out.push(c),
        }
    }
    out.push('\'');
    out
}

/// Backtick-quote an identifier (inner backticks are doubled)
pub fn ident(name: &str) -> String {
    format!("`{}`", name.replace('`', "``"))
}

/// Fully qualified name `db`.`table`
pub fn qualify(db: &str, table: &str) -> String {
    format!("{}.{}", ident(db), ident(table))
}

/// Validate that an identifier is non-empty
pub fn check_ident(name: &str, what: &str) -> ApiResult<()> {
    if name.is_empty() {
        return Err(ApiError::bad(format!("{what}名称不能为空")));
    }
    Ok(())
}

/// Rebuild SQL conditions from the primary key descriptor array returned by the frontend
pub fn build_where(cols: &[String], descs: &[serde_json::Value]) -> ApiResult<String> {
    if cols.len() != descs.len() || cols.is_empty() {
        return Err(ApiError::bad("行定位信息无效"));
    }
    let mut parts = Vec::with_capacity(cols.len());
    for (col, d) in cols.iter().zip(descs.iter()) {
        let t = d.get("t").and_then(|v| v.as_str()).unwrap_or("n");
        let cond = match t {
            "s" => {
                let v = d.get("v").and_then(|v| v.as_str()).unwrap_or("");
                format!("{} = {}", ident(col), sql_string_lit(v))
            }
            "b" => {
                let v = d.get("v").and_then(|v| v.as_str()).unwrap_or("");
                if !is_hex(v) {
                    return Err(ApiError::bad("二进制主键数据无效"));
                }
                format!("{} = UNHEX('{}')", ident(col), v)
            }
            _ => format!("{} IS NULL", ident(col)),
        };
        parts.push(cond);
    }
    Ok(parts.join(" AND "))
}

pub fn is_hex(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Check whether a column data type is binary (handled as hex when decoding, editing, exporting)
pub fn is_binary_type(data_type: &str) -> bool {
    matches!(
        data_type,
        "binary" | "varbinary" | "blob" | "tinyblob" | "mediumblob" | "longblob" | "geometry" | "bit"
    )
}

// ============ SQL statement splitting (for import / console) ============

/// Split SQL text into individual statements by delimiter:
/// - Supports single/double-quoted strings, backtick identifiers, and their escapes
/// - Supports `-- ` and # line comments, standard block comments
/// - /*!40000 ... */ executable comments: shell stripped, treated as normal SQL
/// - Supports the DELIMITER directive (stored procedure/trigger dumps)
pub fn split_sql(input: &str) -> Vec<String> {
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    let b = input.as_bytes();
    let mut stmts = Vec::new();
    let mut cur = String::new();
    let mut i = 0usize;
    let mut delim: Vec<u8> = b";".to_vec();

    while i < b.len() {
        // Recognize the DELIMITER directive after a statement boundary (current buffer is whitespace only)
        if cur.chars().all(|c| c.is_whitespace()) {
            if let Some((new_delim, consumed)) = try_parse_delimiter(&input[i..]) {
                delim = new_delim;
                i += consumed;
                continue;
            }
        }
        let rest = &b[i..];
        match rest[0] {
            b'\'' | b'"' => {
                let q = rest[0] as char;
                cur.push(q);
                i += 1;
                while i < b.len() {
                    let c = b[i] as char;
                    cur.push(c);
                    i += 1;
                    if c == '\\' && i < b.len() {
                        cur.push(b[i] as char);
                        i += 1;
                    } else if (c as u8) == (q as u8) {
                        if i < b.len() && b[i] == (q as u8) {
                            cur.push(q);
                            i += 1;
                        } else {
                            break;
                        }
                    }
                }
            }
            b'`' => {
                cur.push('`');
                i += 1;
                while i < b.len() {
                    let c = b[i] as char;
                    cur.push(c);
                    i += 1;
                    if c == '`' {
                        if i < b.len() && b[i] == b'`' {
                            cur.push('`');
                            i += 1;
                        } else {
                            break;
                        }
                    }
                }
            }
            b'-' if rest.len() > 1 && rest[1] == b'-' => {
                // `-- ` line comment (must be followed by whitespace/newline/EOF so expressions like a--b are not misparsed)
                let next = rest.get(2).copied();
                if next.map_or(true, |c| c == b' ' || c == b'\t' || c == b'\n' || c == b'\r') {
                    while i < b.len() && b[i] != b'\n' {
                        i += 1;
                    }
                } else {
                    cur.push('-');
                    i += 1;
                }
            }
            b'#' => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
            }
            b'/' if rest.len() > 1 && rest[1] == b'*' => {
                if rest.len() > 2 && rest[2] == b'!' {
                    // /*!...*/ executable comment: strip the shell and keep the contents
                    i += 3;
                    while i < b.len() && (b[i] as char).is_ascii_digit() {
                        i += 1;
                    }
                    let start = i;
                    while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                        i += 1;
                    }
                    cur.push_str(&input[start..i.min(b.len())]);
                    i = (i + 2).min(b.len());
                } else {
                    i += 2;
                    while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                        i += 1;
                    }
                    i = (i + 2).min(b.len());
                }
            }
            _ => {
                if rest.len() >= delim.len() && &rest[..delim.len()] == delim.as_slice() {
                    push_stmt(&mut stmts, &cur);
                    cur.clear();
                    i += delim.len();
                } else {
                    let ch_len = utf8_len(rest[0]);
                    cur.push_str(&input[i..i + ch_len]);
                    i += ch_len;
                }
            }
        }
    }
    push_stmt(&mut stmts, &cur);
    stmts
}

fn utf8_len(first: u8) -> usize {
    if first < 0x80 {
        1
    } else if first < 0xE0 {
        2
    } else if first < 0xF0 {
        3
    } else {
        4
    }
}

fn push_stmt(stmts: &mut Vec<String>, cur: &str) {
    let t = cur.trim();
    if !t.is_empty() {
        stmts.push(t.to_string());
    }
}

/// Line-leading DELIMITER directive: DELIMITER ;; or DELIMITER $$
fn try_parse_delimiter(line: &str) -> Option<(Vec<u8>, usize)> {
    let leading = line.len() - line.trim_start().len();
    let head = &line[leading..];
    if head.len() < 9 || !head[..9].to_ascii_lowercase().eq("delimiter") {
        return None;
    }
    let after = &head[9..];
    let nl = after.find('\n');
    let token = after[..nl.unwrap_or(after.len())].trim();
    if !after.starts_with(|ch: char| ch.is_whitespace())
        || token.is_empty()
        || token.contains('\r')
        || token.contains(' ')
        || token.contains('\t')
    {
        return None;
    }
    let consumed = leading + 9 + nl.map(|p| p + 1).unwrap_or(after.len());
    Some((token.as_bytes().to_vec(), consumed))
}

// ============ Statement classification ============

/// Determine whether a statement may return a result set (used to choose the row-fetch path)
pub fn returns_rows(stmt: &str) -> bool {
    let head = stmt.split_whitespace().next().unwrap_or("").to_ascii_lowercase();
    matches!(
        head.as_str(),
        "select" | "show" | "desc" | "describe" | "explain" | "with" | "table" | "analyze" | "call"
    )
}
