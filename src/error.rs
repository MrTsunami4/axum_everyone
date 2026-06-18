use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Validation(#[from] validator::ValidationErrors),
    #[error(transparent)]
    DBError(#[from] toasty::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
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
        }
        .into_response()
    }
}
