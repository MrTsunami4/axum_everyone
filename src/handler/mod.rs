use axum::{debug_handler, extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use tracing::error;

use crate::AppState;

use self::create::add;

mod create;
mod delete;
mod read;

#[derive(Serialize, Deserialize, Debug, FromRow)]
pub struct Joke {
    url: String,
}

#[debug_handler]
pub async fn add_joke(
    State(state): State<AppState>,
    Json(payload): Json<Joke>,
) -> Result<(StatusCode, Json<Joke>), StatusCode> {
    let res = add(state, payload).await;
    match res {
        Ok(joke) => Ok((StatusCode::CREATED, Json(joke))),
        Err(result) => {
            error!("Error inserting joke: {:?}", result);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[debug_handler]
pub async fn delete_all_joke(State(state): State<AppState>) -> StatusCode {
    let result = delete::remove(state).await;
    match result {
        Ok(row_affected) if row_affected > 0 => StatusCode::OK,
        Ok(_) => StatusCode::NOT_MODIFIED,
        Err(err) => {
            error!("Error deleting jokes: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[debug_handler]
pub async fn get_random_joke(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<Joke>), StatusCode> {
    let row = read::get_random_joke(state).await;
    match row {
        Ok(Some(joke)) => Ok((StatusCode::OK, Json(joke))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Error getting joke: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
