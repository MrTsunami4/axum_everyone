use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use tracing::instrument;
use validator::Validate;

use crate::{
    error::AppError, request::user_request::UserRequest, schemas::user::User, state::AppState,
};

#[instrument(skip(state))]
pub async fn add_user(
    State(mut state): State<AppState>,
    Json(payload): Json<UserRequest>,
) -> Result<(StatusCode, Json<User>), AppError> {
    payload.validate()?;
    let user = toasty::create!(User {
        name: payload.name,
        email: payload.email,
    })
    .exec(&mut state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(user)))
}

#[instrument(skip(state))]
pub async fn update_user(
    Path(id): Path<i64>,
    State(mut state): State<AppState>,
    Json(payload): Json<UserRequest>,
) -> Result<StatusCode, AppError> {
    payload.validate()?;
    toasty::update!(User::filter_by_id(id) {
        name: payload.name,
        email: payload.email,
    })
    .exec(&mut state.db)
    .await?;
    Ok(StatusCode::OK)
}

#[instrument(skip(state))]
pub async fn delete_all_users(State(mut state): State<AppState>) -> Result<StatusCode, AppError> {
    User::all().delete().exec(&mut state.db).await?;
    Ok(StatusCode::OK)
}

#[instrument(skip(state))]
pub async fn get_user(
    Path(id): Path<i64>,
    State(mut state): State<AppState>,
) -> Result<Json<User>, AppError> {
    let user = User::get_by_id(&mut state.db, id).await?;
    Ok(Json(user))
}

#[instrument(skip(state))]
pub async fn get_all_users(State(mut state): State<AppState>) -> Result<Json<Vec<User>>, AppError> {
    let users = User::all()
        .order_by(User::fields().id().asc())
        .exec(&mut state.db)
        .await?;
    Ok(Json(users))
}

#[instrument(skip(state))]
pub async fn delete_user(
    Path(id): Path<i64>,
    State(mut state): State<AppState>,
) -> Result<StatusCode, AppError> {
    User::delete_by_id(&mut state.db, id).await?;
    Ok(StatusCode::OK)
}
