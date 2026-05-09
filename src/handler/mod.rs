mod create;
mod delete;
mod read;
mod update;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tracing::error;
use validator::Validate;

use std::error::Error;

use crate::models::{
    AppState, CreateJokeRequest, ErrorResponse, Joke, JokeQueryParams, PaginatedJokesResponse,
    UpdateJokeRequest,
};

/// Application-level error type with HTTP status mapping.
#[derive(Debug)]
pub enum AppError {
    NotFound,
    Validation(String),
    Internal,
}

impl AppError {}

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
        (status, Json(ErrorResponse { error: msg })).into_response()
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        Self::Validation(validation_msg(&err))
    }
}

/// Log the error and return an internal server error.
fn log_and_internal<E: Error>(context: &str, err: E) -> AppError {
    error!("{}: {:?}", context, err);
    AppError::Internal
}

/// Extract validation error messages into a joined string.
fn validation_msg(err: &validator::ValidationErrors) -> String {
    err.field_errors()
        .values()
        .flat_map(|field_errors| {
            field_errors
                .iter()
                .filter_map(|e| e.message.as_ref().map(std::string::ToString::to_string))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Create a new joke.
///
/// # Errors
///
/// Returns `AppError::Validation` if the content is invalid,
/// or `AppError::Internal` on database failure.
pub async fn add_joke(
    State(state): State<AppState>,
    Json(payload): Json<CreateJokeRequest>,
) -> Result<(StatusCode, Json<Joke>), AppError> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(validation_msg(&e)))?;

    create::add(&payload, &state.db)
        .await
        .map(|joke| (StatusCode::CREATED, Json(joke)))
        .map_err(|err| log_and_internal("Error inserting joke", err))
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
    Json(payload): Json<UpdateJokeRequest>,
) -> Result<Json<Joke>, AppError> {
    payload
        .validate()
        .map_err(|e| AppError::Validation(validation_msg(&e)))?;

    update::update(id, &payload, &state.db)
        .await
        .map_err(|err| log_and_internal("Error updating joke", err))
        .and_then(|opt| opt.map_or(Err(AppError::NotFound), |joke| Ok(Json(joke))))
}

/// Delete all jokes.
///
/// # Errors
///
/// Returns `AppError::Internal` on database failure.
pub async fn delete_all_jokes(State(state): State<AppState>) -> Result<StatusCode, AppError> {
    delete::remove(&state.db)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|err| log_and_internal("Error deleting jokes", err))
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
    read::get_joke(id, &state.db)
        .await
        .map_err(|err| log_and_internal("Error getting joke", err))
        .and_then(|opt| opt.map_or(Err(AppError::NotFound), |joke| Ok(Json(joke))))
}

/// Get all jokes with pagination.
///
/// # Errors
///
/// Returns `AppError::Internal` on database failure.
pub async fn get_all_jokes(
    State(state): State<AppState>,
    Query(params): Query<JokeQueryParams>,
) -> Result<Json<PaginatedJokesResponse>, AppError> {
    let page = params.page();
    let per_page = params.per_page();
    let offset = params.offset();

    let jokes = read::get_all_jokes(&state.db, per_page, offset)
        .await
        .map_err(|err| log_and_internal("Error getting jokes", err))?;

    let total = read::count_jokes(&state.db)
        .await
        .map_err(|err| log_and_internal("Error counting jokes", err))?;

    let total_pages = if per_page > 0 {
        (total + per_page - 1) / per_page
    } else {
        0
    };

    Ok(Json(PaginatedJokesResponse {
        jokes,
        total,
        page,
        per_page,
        total_pages,
    }))
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
    delete::delete_joke(id, &state.db)
        .await
        .map_err(|err| log_and_internal("Error deleting joke", err))
        .and_then(|rows| {
            if rows > 0 {
                Ok(StatusCode::OK)
            } else {
                Err(AppError::NotFound)
            }
        })
}

/// Get a random joke.
///
/// # Errors
///
/// Returns `AppError::NotFound` if there are no jokes,
/// or `AppError::Internal` on database failure.
pub async fn get_random_joke(State(state): State<AppState>) -> Result<Json<Joke>, AppError> {
    read::get_random_joke(&state.db)
        .await
        .map_err(|err| log_and_internal("Error getting random joke", err))
        .and_then(|opt| opt.map_or(Err(AppError::NotFound), |joke| Ok(Json(joke))))
}
