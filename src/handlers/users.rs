use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use tracing::instrument;

use crate::{
    error::AppError,
    request::{ValidatedJson, user_request::UserRequest},
    schemas::user::User,
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/users",
    tag = "Users",
    request_body = UserRequest,
    responses(
        (status = 201, description = "User created", body = User),
        (status = 400, description = "Validation error", body = String),
    ),
)]
#[instrument(skip(state))]
pub async fn add_user(
    State(mut state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<UserRequest>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let user = toasty::create!(User {
        name: payload.name,
        email: payload.email,
    })
    .exec(&mut state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(user)))
}

#[utoipa::path(
    put,
    path = "/user/{id}",
    tag = "Users",
    request_body = UserRequest,
    params(
        ("id" = i64, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "User updated"),
        (status = 400, description = "Validation error", body = String),
    ),
)]
#[instrument(skip(state))]
pub async fn update_user(
    Path(id): Path<i64>,
    State(mut state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<UserRequest>,
) -> Result<StatusCode, AppError> {
    toasty::update!(User::filter_by_id(id) {
        name: payload.name,
        email: payload.email,
    })
    .exec(&mut state.db)
    .await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    delete,
    path = "/users",
    tag = "Users",
    responses((status = 200, description = "All users deleted")),
)]
#[instrument(skip(state))]
pub async fn delete_all_users(State(mut state): State<AppState>) -> Result<StatusCode, AppError> {
    User::all().delete().exec(&mut state.db).await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    get,
    path = "/user/{id}",
    tag = "Users",
    params(
        ("id" = i64, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found", body = String),
    ),
)]
#[instrument(skip(state))]
pub async fn get_user(
    Path(id): Path<i64>,
    State(mut state): State<AppState>,
) -> Result<Json<User>, AppError> {
    let user = User::get_by_id(&mut state.db, id).await?;
    Ok(Json(user))
}

#[utoipa::path(
    get,
    path = "/users",
    tag = "Users",
    responses((status = 200, description = "List of all users", body = Vec<User>)),
)]
#[instrument(skip(state))]
pub async fn get_all_users(State(mut state): State<AppState>) -> Result<Json<Vec<User>>, AppError> {
    let users = User::all()
        .order_by(User::fields().id().asc())
        .exec(&mut state.db)
        .await?;
    Ok(Json(users))
}

#[utoipa::path(
    delete,
    path = "/user/{id}",
    tag = "Users",
    params(
        ("id" = i64, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "User deleted"),
        (status = 404, description = "User not found", body = String),
    ),
)]
#[instrument(skip(state))]
pub async fn delete_user(
    Path(id): Path<i64>,
    State(mut state): State<AppState>,
) -> Result<StatusCode, AppError> {
    User::delete_by_id(&mut state.db, id).await?;
    Ok(StatusCode::OK)
}
