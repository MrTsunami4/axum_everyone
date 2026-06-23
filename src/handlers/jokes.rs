use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use validator::Validate;

use crate::{
    error::AppError, request::joke_request::JokeRequest, schemas::joke::Joke, schemas::user::User,
    state::AppState,
};

pub async fn add_joke(
    Path(user_id): Path<i64>,
    State(mut state): State<AppState>,
    Json(payload): Json<JokeRequest>,
) -> Result<(StatusCode, Json<Joke>), AppError> {
    payload.validate()?;
    let user = User::get_by_id(&mut state.db, user_id).await?;
    let joke = toasty::create!(in user.jokes() {
        content: payload.content
    })
    .exec(&mut state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(joke)))
}

pub async fn update_joke(
    Path(id): Path<i64>,
    State(mut state): State<AppState>,
    Json(payload): Json<JokeRequest>,
) -> Result<StatusCode, AppError> {
    payload.validate()?;
    toasty::update!(Joke::filter_by_id(id) { content: payload.content })
        .exec(&mut state.db)
        .await?;
    Ok(StatusCode::OK)
}

pub async fn delete_all_jokes(State(mut state): State<AppState>) -> Result<StatusCode, AppError> {
    Joke::all().delete().exec(&mut state.db).await?;
    Ok(StatusCode::OK)
}

pub async fn get_joke(
    Path(id): Path<i64>,
    State(mut state): State<AppState>,
) -> Result<Json<Joke>, AppError> {
    let joke = Joke::get_by_id(&mut state.db, id).await?;
    Ok(Json(joke))
}

pub async fn get_all_jokes(State(mut state): State<AppState>) -> Result<Json<Vec<Joke>>, AppError> {
    let jokes = Joke::all()
        .order_by(Joke::fields().id().asc())
        .exec(&mut state.db)
        .await?;
    Ok(Json(jokes))
}

pub async fn get_user_jokes(
    Path(user_id): Path<i64>,
    State(mut state): State<AppState>,
) -> Result<Json<Vec<Joke>>, AppError> {
    let user = User::get_by_id(&mut state.db, user_id).await?;
    let jokes = user.jokes().exec(&mut state.db).await?;
    Ok(Json(jokes))
}

pub async fn delete_joke(
    Path(id): Path<i64>,
    State(mut state): State<AppState>,
) -> Result<StatusCode, AppError> {
    Joke::delete_by_id(&mut state.db, id).await?;
    Ok(StatusCode::OK)
}
