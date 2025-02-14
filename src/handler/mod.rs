use axum::{
    debug_handler,
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use deadpool_diesel::sqlite::Pool;
use tracing::error;

use crate::{
    internal_error,
    models::{Joke, NewJoke},
};

mod create;
mod delete;
mod read;

#[debug_handler]
pub async fn add_joke(
    State(pool): State<Pool>,
    Json(payload): Json<NewJoke>,
) -> Result<(StatusCode, Json<Joke>), StatusCode> {
    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn.interact(|conn| create::add(payload, conn)).await;
    match res {
        Ok(Ok(joke)) => Ok((StatusCode::CREATED, Json(joke))),
        Ok(Err(err)) => {
            error!("Error inserting joke: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(result) => {
            error!("Error connecting to the db: {:?}", result);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[debug_handler]
pub async fn delete_all_joke(State(pool): State<Pool>) -> Result<StatusCode, StatusCode> {
    let conn = pool.get().await.map_err(internal_error)?;
    let row_accected = conn.interact(|conn| delete::remove(conn)).await;
    match row_accected {
        Ok(Ok(row_affected)) => {
            if row_affected > 0 {
                Ok(StatusCode::OK)
            } else {
                Ok(StatusCode::NOT_MODIFIED)
            }
        }
        Ok(Err(err)) => {
            error!("Error deleting jokes: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(result) => {
            error!("Error connecting to the db: {:?}", result);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[debug_handler]
pub async fn get_joke(
    Path(id): Path<i32>,
    State(pool): State<Pool>,
) -> Result<(StatusCode, Json<Joke>), StatusCode> {
    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn.interact(move |conn| read::get_joke(id, conn)).await;
    match res {
        Ok(Ok(Some(joke))) => Ok((StatusCode::OK, Json(joke))),
        Ok(Ok(None)) => Err(StatusCode::NOT_FOUND),
        Ok(Err(err)) => {
            error!("Error getting joke: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(result) => {
            error!("Error connecting to the db: {:?}", result);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[debug_handler]
pub async fn get_all_jokes(
    State(pool): State<Pool>,
) -> Result<(StatusCode, Json<Vec<Joke>>), StatusCode> {
    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn.interact(|conn| read::get_all_jokes(conn)).await;
    match res {
        Ok(Ok(jokes)) => Ok((StatusCode::OK, Json(jokes))),
        Ok(Err(err)) => {
            error!("Error getting jokes: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(result) => {
            error!("Error connecting to the db: {:?}", result);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[debug_handler]
pub async fn delete_joke(
    Path(id): Path<i32>,
    State(pool): State<Pool>,
) -> Result<StatusCode, StatusCode> {
    let conn = pool.get().await.map_err(internal_error)?;
    let res = conn
        .interact(move |conn| delete::delete_joke(id, conn))
        .await;
    match res {
        Ok(Ok(_)) => Ok(StatusCode::NO_CONTENT),
        Ok(Err(err)) => {
            error!("Error deleting joke: {:?}", err);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
        Err(result) => {
            error!("Error connecting to the db: {:?}", result);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
