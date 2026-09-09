use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use tracing::instrument;

use crate::{
    error::{AppError, ErrorResponse},
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
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 409, description = "User with this email already exists", body = ErrorResponse),
    ),
)]
#[instrument(skip(state))]
pub async fn add_user(
    State(mut state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<UserRequest>,
) -> Result<(StatusCode, Json<User>), AppError> {
    let inserted = User::upsert_by_email(payload.email.clone())
        .on_create(|user| user.name(payload.name.clone()))
        .or_ignore()
        .exec(&mut state.db)
        .await?;

    match inserted {
        Some(user) => Ok((StatusCode::CREATED, Json(user))),
        None => Err(AppError::Conflict(
            "A user with this email already exists".to_string(),
        )),
    }
}

#[utoipa::path(
    put,
    path = "/users/{id}",
    tag = "Users",
    request_body = UserRequest,
    params(
        ("id" = i64, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "User updated"),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 409, description = "Email already in use", body = ErrorResponse),
    ),
)]
#[instrument(skip(state))]
pub async fn update_user(
    Path(id): Path<i64>,
    State(mut state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<UserRequest>,
) -> Result<StatusCode, AppError> {
    // Existence check: `toasty::update!` is a silent no-op when no row matches.
    let user = User::get_by_id(&mut state.db, id).await?;
    // Uniqueness check: a unique-index violation would otherwise surface as a 500.
    let matches = User::filter(User::fields().email().eq(&payload.email))
        .exec(&mut state.db)
        .await?;
    if matches.iter().any(|existing| existing.id != user.id) {
        return Err(AppError::Conflict("Email is already in use".to_string()));
    }
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
    responses((status = 204, description = "All users deleted")),
)]
#[instrument(skip(state))]
pub async fn delete_all_users(
    State(mut state): State<AppState>,
) -> Result<StatusCode, AppError> {
    // Upstream bug workaround: `User::all().delete()` panics in toasty 0.10
    // while lowering the has_many cascade
    // (engine/lower.rs cast_expr hits todo!() for a column operand; still
    // unfixed on toasty main). Deleting per-id skips relation planning.
    let users = User::all().exec(&mut state.db).await?;
    for id in users.iter().map(|user| user.id) {
        User::delete_by_id(&mut state.db, id).await?;
    }
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    tag = "Users",
    params(
        ("id" = i64, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "User found", body = User),
        (status = 404, description = "User not found", body = ErrorResponse),
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
    path = "/users/{id}",
    tag = "Users",
    params(
        ("id" = i64, Path, description = "User ID"),
    ),
    responses(
        (status = 204, description = "User deleted"),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
)]
#[instrument(skip(state))]
pub async fn delete_user(
    Path(id): Path<i64>,
    State(mut state): State<AppState>,
) -> Result<StatusCode, AppError> {
    // Existence check: deleting a missing user should 404, not silently succeed.
    User::get_by_id(&mut state.db, id).await?;
    User::delete_by_id(&mut state.db, id).await?;
    Ok(StatusCode::NO_CONTENT)
}
