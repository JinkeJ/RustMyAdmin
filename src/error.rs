use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, msg: impl Into<String>) -> Self {
        Self { status, message: msg.into() }
    }
    pub fn bad(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, msg)
    }
    pub fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "未登录或会话已失效")
    }
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, msg)
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, msg)
    }
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, msg)
    }
    pub fn connection_lost(msg: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_GATEWAY, msg)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self {
        match &e {
            sqlx::Error::Database(db) => {
                let msg = db.message().to_string();
                let code = db.code();
                match code.as_deref().unwrap_or("") {
                    "1044" | "1142" | "1143" | "1227" | "1698" => ApiError::forbidden(msg),
                    "1045" | "1049" | "2003" | "2005" => ApiError::connection_lost(msg),
                    "1064" => ApiError::bad(format!("SQL 语法错误: {msg}")),
                    "1146" => ApiError::not_found(msg),
                    _ => ApiError::bad(msg),
                }
            }
            sqlx::Error::Io(_) | sqlx::Error::Protocol(_) | sqlx::Error::WorkerCrashed => {
                ApiError::connection_lost(format!("数据库连接错误: {e}"))
            }
            _ => ApiError::internal(e.to_string()),
        }
    }
}
