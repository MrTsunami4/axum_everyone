mod create;
mod delete;
mod read;
mod update;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;
use validator::Validate;

use std::error::Error;

use crate::models::{AppState, Joke, JokeRequest};

/// Application-level error type with HTTP status mapping.
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
        log_and_internal("Database error", err)
    }
}

/// Log the error and return an internal server error.
fn log_and_internal<E: Error>(context: &str, err: E) -> AppError {
    error!("{}: {:?}", context, err);
    AppError::Internal
}

/// Create a new joke.
///
/// # Errors
///
/// Returns `AppError::Validation` if the content is invalid,
/// or `AppError::Internal` on database failure.
pub async fn add_joke(
    State(state): State<AppState>,
    Json(payload): Json<JokeRequest>,
) -> Result<(StatusCode, Json<Joke>), AppError> {
    payload.validate()?;
    let mut db = state.db.clone();
    let joke = create::add(&payload.content, &mut db).await?;
    Ok((StatusCode::CREATED, Json(joke)))
}

/// Update an existing joke by ID.
///
/// # Errors
///
/// Returns `AppError::NotFound` if the joke doesn't exist,
/// `AppError::Validation` if the content is invalid,
/// or `AppError::Internal` on database failure.
pub async fn update_joke(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(payload): Json<JokeRequest>,
) -> Result<Json<Joke>, AppError> {
    payload.validate()?;
    let mut db = state.db.clone();
    let joke = update::update(id, &payload.content, &mut db)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(joke))
}

/// Delete all jokes.
///
/// # Errors
///
/// Returns `AppError::Internal` on database failure.
pub async fn delete_all_jokes(State(state): State<AppState>) -> Result<StatusCode, AppError> {
    let mut db = state.db.clone();
    delete::remove(&mut db).await?;
    Ok(StatusCode::OK)
}

/// Get a joke by ID.
///
/// # Errors
///
/// Returns `AppError::NotFound` if the joke doesn't exist,
/// or `AppError::Internal` on database failure.
pub async fn get_joke(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<Joke>, AppError> {
    let mut db = state.db.clone();
    let joke = read::get_joke(id, &mut db)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(joke))
}

/// Get all jokes with pagination.
///
/// # Errors
///
/// Returns `AppError::Internal` on database failure.
pub async fn get_all_jokes(State(state): State<AppState>) -> Result<Json<Vec<Joke>>, AppError> {
    let mut db = state.db.clone();
    let jokes = read::get_all_jokes(&mut db).await?;
    Ok(Json(jokes))
}

/// Delete a joke by ID.
///
/// # Errors
///
/// Returns `AppError::NotFound` if the joke doesn't exist,
/// or `AppError::Internal` on database failure.
pub async fn delete_joke(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    let mut db = state.db.clone();
    delete::delete_joke(id, &mut db).await?;
    Ok(StatusCode::OK)
}
