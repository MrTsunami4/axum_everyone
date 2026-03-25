use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sqlx::SqlitePool;
use tracing::error;

use std::error::Error;

use crate::models::{ErrorResponse, Joke, NewJoke};

mod create;
mod delete;
mod read;

const MAX_JOKE_LENGTH: usize = 1000;

pub enum AppError {
    NotFound,
    BadRequest(String),
    Internal,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            Self::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
        };
        (status, Json(ErrorResponse { error: msg })).into_response()
    }
}

fn log_and_internal<E: Error>(context: &str, err: E) -> AppError {
    error!("{}: {:?}", context, err);
    AppError::Internal
}

pub async fn add_joke(
    State(pool): State<SqlitePool>,
    Json(payload): Json<NewJoke>,
) -> Result<(StatusCode, Json<Joke>), AppError> {
    if payload.content.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Joke content cannot be empty".to_string(),
        ));
    }
    if payload.content.len() > MAX_JOKE_LENGTH {
        return Err(AppError::BadRequest(format!(
            "Joke content cannot exceed {MAX_JOKE_LENGTH} characters"
        )));
    }
    create::add(&payload, &pool)
        .await
        .map(|joke| (StatusCode::CREATED, Json(joke)))
        .map_err(|err| log_and_internal("Error inserting joke", err))
}

pub async fn delete_all_joke(State(pool): State<SqlitePool>) -> Result<StatusCode, AppError> {
    delete::remove(&pool)
        .await
        .map(|_| StatusCode::OK)
        .map_err(|err| log_and_internal("Error deleting jokes", err))
}

pub async fn get_joke(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Joke>, AppError> {
    read::get_joke(id, &pool)
        .await
        .map_err(|err| log_and_internal("Error getting joke", err))
        .and_then(|opt| opt.map_or(Err(AppError::NotFound), |joke| Ok(Json(joke))))
}

pub async fn get_all_jokes(State(pool): State<SqlitePool>) -> Result<Json<Vec<Joke>>, AppError> {
    read::get_all_jokes(&pool)
        .await
        .map(Json)
        .map_err(|err| log_and_internal("Error getting jokes", err))
}

pub async fn delete_joke(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<StatusCode, AppError> {
    delete::delete_joke(id, &pool)
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

pub async fn get_random_joke(State(pool): State<SqlitePool>) -> Result<Json<Joke>, AppError> {
    read::get_random_joke(&pool)
        .await
        .map_err(|err| log_and_internal("Error getting random joke", err))
        .and_then(|opt| opt.map_or(Err(AppError::NotFound), |joke| Ok(Json(joke))))
}
