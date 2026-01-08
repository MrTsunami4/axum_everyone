use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use sqlx::SqlitePool;
use tracing::error;

use std::error::Error;

use crate::models::{Joke, NewJoke};

mod create;
mod delete;
mod read;

fn log_and_internal<E: Error>(context: &str, err: E) -> StatusCode {
    error!("{}: {:?}", context, err);
    StatusCode::INTERNAL_SERVER_ERROR
}

const fn modified_status(rows_affected: u64) -> StatusCode {
    if rows_affected > 0 {
        StatusCode::OK
    } else {
        StatusCode::NOT_MODIFIED
    }
}

pub async fn add_joke(
    State(pool): State<SqlitePool>,
    Json(payload): Json<NewJoke>,
) -> Result<(StatusCode, Json<Joke>), StatusCode> {
    create::add(&payload, &pool)
        .await
        .map(|joke| (StatusCode::CREATED, Json(joke)))
        .map_err(|err| log_and_internal("Error inserting joke", err))
}

pub async fn delete_all_joke(State(pool): State<SqlitePool>) -> Result<StatusCode, StatusCode> {
    delete::remove(&pool)
        .await
        .map(modified_status)
        .map_err(|err| log_and_internal("Error deleting jokes", err))
}

pub async fn get_joke(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<(StatusCode, Json<Joke>), StatusCode> {
    read::get_joke(id, &pool)
        .await
        .map_err(|err| log_and_internal("Error getting joke", err))
        .and_then(|opt_joke| {
            opt_joke.map_or(Err(StatusCode::NOT_FOUND), |joke| {
                Ok((StatusCode::OK, Json(joke)))
            })
        })
}

pub async fn get_all_jokes(
    State(pool): State<SqlitePool>,
) -> Result<(StatusCode, Json<Vec<Joke>>), StatusCode> {
    read::get_all_jokes(&pool)
        .await
        .map(|jokes| (StatusCode::OK, Json(jokes)))
        .map_err(|err| log_and_internal("Error getting jokes", err))
}

pub async fn delete_joke(
    Path(id): Path<i64>,
    State(pool): State<SqlitePool>,
) -> Result<StatusCode, StatusCode> {
    delete::delete_joke(id, &pool)
        .await
        .map(modified_status)
        .map_err(|err| log_and_internal("Error deleting joke", err))
}
