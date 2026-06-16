use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use validator::Validate;

use crate::{
    error::AppError, request::joke_request::JokeRequest, schemas::joke::Joke, state::AppState,
};

pub async fn add_joke(
    State(state): State<AppState>,
    Json(payload): Json<JokeRequest>,
) -> Result<(StatusCode, Json<Joke>), AppError> {
    payload.validate()?;
    let mut db = state.db.clone();
    let joke = toasty::create!(Joke {
        content: payload.content
    })
    .exec(&mut db)
    .await?;
    Ok((StatusCode::CREATED, Json(joke)))
}

pub async fn update_joke(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(payload): Json<JokeRequest>,
) -> Result<Json<Joke>, AppError> {
    payload.validate()?;
    let mut db = state.db.clone();
    let _joke = Joke::get_by_id(&mut db, id).await?;
    toasty::update!(Joke::filter_by_id(id) { content: payload.content })
        .exec(&mut db)
        .await?;
    let joke = Joke::get_by_id(&mut db, id).await?;
    Ok(Json(joke))
}

pub async fn delete_all_jokes(State(state): State<AppState>) -> Result<StatusCode, AppError> {
    let mut db = state.db.clone();
    Joke::all().delete().exec(&mut db).await?;
    Ok(StatusCode::OK)
}

pub async fn get_joke(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<Joke>, AppError> {
    let mut db = state.db.clone();
    let joke = Joke::get_by_id(&mut db, id).await?;
    Ok(Json(joke))
}

pub async fn get_all_jokes(State(state): State<AppState>) -> Result<Json<Vec<Joke>>, AppError> {
    let mut db = state.db.clone();
    let jokes = Joke::all()
        .order_by(Joke::fields().id().asc())
        .exec(&mut db)
        .await?;
    Ok(Json(jokes))
}

pub async fn delete_joke(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    let mut db = state.db.clone();
    Joke::delete_by_id(&mut db, id).await?;
    Ok(StatusCode::OK)
}
