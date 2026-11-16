use axum::{Json, extract::State, http::StatusCode};
use tracing::instrument;

use crate::{
    SerializablePage,
    error::{AppError, ErrorResponse},
    request::{
        ApiPath, ValidatedJson, ValidatedQuery,
        joke_request::{JokeRequest, PaginationParams},
    },
    schemas::{joke::Joke, user::User},
    state::AppState,
};

#[utoipa::path(
    post,
    path = "/users/{user_id}/jokes",
    tag = "Jokes",
    request_body = JokeRequest,
    params(
        ("user_id" = i64, Path, description = "User ID"),
    ),
    responses(
        (status = 201, description = "Joke created", body = Joke),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
)]
#[instrument(skip_all)]
pub async fn add_joke(
    ApiPath(user_id): ApiPath<i64>,
    State(mut state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<JokeRequest>,
) -> Result<(StatusCode, Json<Joke>), AppError> {
    let mut tx = state.db.transaction().await?;
    let user = User::get_by_id(&mut tx, user_id).await?;
    let joke = toasty::create!(in user.jokes() {
        content: payload.content
    })
    .exec(&mut tx)
    .await?;
    tx.commit().await?;
    Ok((StatusCode::CREATED, Json(joke)))
}

#[utoipa::path(
    get,
    path = "/users/{user_id}/jokes",
    tag = "Jokes",
    params(
        ("user_id" = i64, Path, description = "User ID"),
    ),
    responses(
        (status = 200, description = "List of jokes for the user", body = Vec<Joke>),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
)]
#[instrument(skip_all)]
pub async fn get_user_jokes(
    ApiPath(user_id): ApiPath<i64>,
    State(mut state): State<AppState>,
) -> Result<Json<Vec<Joke>>, AppError> {
    // 404 on a missing user, mirroring add_joke.
    User::get_by_id(&mut state.db, user_id).await?;
    let jokes = Joke::filter_by_user_id(user_id).exec(&mut state.db).await?;
    Ok(Json(jokes))
}

#[utoipa::path(
    get,
    path = "/jokes",
    tag = "Jokes",
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated jokes", body = SerializablePage<Joke>),
    ),
)]
#[instrument(skip_all)]
pub async fn list_jokes(
    State(mut state): State<AppState>,
    ValidatedQuery(params): ValidatedQuery<PaginationParams>,
) -> Result<Json<SerializablePage<Joke>>, AppError> {
    let query = Joke::all()
        .order_by(Joke::fields().id().asc())
        .paginate(params.page_size.unwrap_or(10));
    let query = match params.cursor {
        Some(cursor) => query.after(cursor),
        None => query,
    };
    let page: SerializablePage<Joke> = query.exec(&mut state.db).await?.into();
    Ok(Json(page))
}

#[utoipa::path(
    put,
    path = "/jokes/{id}",
    tag = "Jokes",
    request_body = JokeRequest,
    params(
        ("id" = i64, Path, description = "Joke ID"),
    ),
    responses(
        (status = 200, description = "Joke updated"),
        (status = 400, description = "Validation error", body = ErrorResponse),
        (status = 404, description = "Joke not found", body = ErrorResponse),
    ),
)]
#[instrument(skip_all)]
pub async fn update_joke(
    ApiPath(id): ApiPath<i64>,
    State(mut state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<JokeRequest>,
) -> Result<StatusCode, AppError> {
    let mut tx = state.db.transaction().await?;
    // Existence check: the update builder is a silent no-op when no row matches.
    Joke::get_by_id(&mut tx, id).await?;
    Joke::update_by_id(id)
        .content(payload.content)
        .exec(&mut tx)
        .await?;
    tx.commit().await?;
    Ok(StatusCode::OK)
}

#[utoipa::path(
    delete,
    path = "/jokes",
    tag = "Jokes",
    responses((status = 204, description = "All jokes deleted")),
)]
#[instrument(skip_all)]
pub async fn delete_all_jokes(State(mut state): State<AppState>) -> Result<StatusCode, AppError> {
    Joke::all().delete().exec(&mut state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/jokes/{id}",
    tag = "Jokes",
    params(
        ("id" = i64, Path, description = "Joke ID"),
    ),
    responses(
        (status = 200, description = "Joke found", body = Joke),
        (status = 404, description = "Joke not found", body = ErrorResponse),
    ),
)]
#[instrument(skip_all)]
pub async fn get_joke(
    ApiPath(id): ApiPath<i64>,
    State(mut state): State<AppState>,
) -> Result<Json<Joke>, AppError> {
    let joke = Joke::get_by_id(&mut state.db, id).await?;
    Ok(Json(joke))
}

#[utoipa::path(
    delete,
    path = "/jokes/{id}",
    tag = "Jokes",
    params(
        ("id" = i64, Path, description = "Joke ID"),
    ),
    responses(
        (status = 204, description = "Joke deleted"),
        (status = 404, description = "Joke not found", body = ErrorResponse),
    ),
)]
#[instrument(skip_all)]
pub async fn delete_joke(
    ApiPath(id): ApiPath<i64>,
    State(mut state): State<AppState>,
) -> Result<StatusCode, AppError> {
    let mut tx = state.db.transaction().await?;
    // Existence check: deleting a missing joke should 404, not silently succeed.
    Joke::get_by_id(&mut tx, id).await?;
    Joke::delete_by_id(&mut tx, id).await?;
    tx.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}
