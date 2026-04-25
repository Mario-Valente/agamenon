use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Schema not found")]
    NotFound,

    #[error("Schema already exists")]
    AlreadyExists,

    #[error("Invalid schema: {0}")]
    InvalidSchema(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Unexpected error: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum CompatibilityError {
    #[error("Invalid schema: {0}")]
    InvalidSchema(String),

    #[error("Incompatible schema: {0}")]
    Incompatible(String),
}

impl IntoResponse for StorageError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            StorageError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            StorageError::AlreadyExists => (StatusCode::CONFLICT, self.to_string()),
            StorageError::InvalidSchema(ref s) => (StatusCode::BAD_REQUEST, s.clone()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        (status, Json(json!({"error": message}))).into_response()
    }
}

impl IntoResponse for CompatibilityError {
    fn into_response(self) -> Response {
        let message = self.to_string();
        let status = match self {
            CompatibilityError::InvalidSchema(_) => StatusCode::BAD_REQUEST,
            CompatibilityError::Incompatible(_) => StatusCode::CONFLICT,
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}
