use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    DatabaseError(String),
    NotFound(String),
    ValidationError(String),
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": status.as_u16(),
            "message": error_message,
        }));

        (status, body).into_response()
    }
}

// Implement From<surrealdb::Error> for easier conversion
impl From<surrealdb::Error> for AppError {
    fn from(err: surrealdb::Error) -> Self {
        AppError::DatabaseError(format!("Database error: {}", err))
    }
}

// Implement From<anyhow::Error> if needed, or generic strings
impl From<String> for AppError {
    fn from(err: String) -> Self {
        AppError::InternalServerError(err)
    }
}
