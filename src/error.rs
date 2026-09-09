use axum::{
    Json,
    extract::rejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::error;
use utoipa::ToSchema;

/// Standardized JSON error response structure.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub message: String,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Validation(#[from] validator::ValidationErrors),
    #[error(transparent)]
    DBError(#[from] toasty::Error),
    #[error(transparent)]
    JsonError(#[from] rejection::JsonRejection),
    #[error(transparent)]
    QueryError(#[from] rejection::QueryRejection),
    #[error("{0}")]
    Conflict(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Validation(err) => (StatusCode::BAD_REQUEST, err.to_string()),
            Self::DBError(err) if err.is_record_not_found() => {
                (StatusCode::NOT_FOUND, "Not found".to_string())
            }
            Self::DBError(err) => {
                error!("{err:?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                )
            }
            Self::JsonError(err) => (StatusCode::BAD_REQUEST, err.to_string()),
            Self::QueryError(err) => (StatusCode::BAD_REQUEST, err.to_string()),
            Self::Conflict(message) => (StatusCode::CONFLICT, message),
        };

        (status, Json(ErrorResponse { message })).into_response()
    }
}
