use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;

use std::error::Error;

#[derive(Debug)]
pub enum AppError {
    NotFound,
    Validation(String),
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            Self::Validation(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };
        (status, Json(msg)).into_response()
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        Self::Validation(err.to_string())
    }
}

impl From<toasty::Error> for AppError {
    fn from(err: toasty::Error) -> Self {
        if err.is_record_not_found() {
            Self::NotFound
        } else {
            log_and_internal("Database error", err)
        }
    }
}

fn log_and_internal<E: Error>(context: &str, err: E) -> AppError {
    error!("{}: {:?}", context, err);
    AppError::Internal
}
