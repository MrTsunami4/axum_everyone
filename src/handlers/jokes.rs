use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use tracing::instrument;

use crate::{
    SerializablePage,
    error::AppError,
    request::{
        ValidatedJson,
        joke_request::{JokeRequest, PaginationParams},
    },
    schemas::{joke::Joke, user::User},
    state::AppState,
};

#[instrument(skip(state))]
pub async fn add_joke(
    Path(user_id): Path<i64>,
    State(mut state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<JokeRequest>,
) -> Result<(StatusCode, Json<Joke>), AppError> {
    let user = User::get_by_id(&mut state.db, user_id).await?;
    let joke = toasty::create!(in user.jokes() {
        content: payload.content
    })
    .exec(&mut state.db)
    .await?;
    Ok((StatusCode::CREATED, Json(joke)))
}

#[instrument(skip(state))]
pub async fn update_joke(
    Path(id): Path<i64>,
    State(mut state): State<AppState>,
    ValidatedJson(payload): ValidatedJson<JokeRequest>,
) -> Result<StatusCode, AppError> {
    Joke::update_by_id(id)
        .content(payload.content)
        .exec(&mut state.db)
        .await?;
    Ok(StatusCode::OK)
}

#[instrument(skip(state))]
pub async fn delete_all_jokes(State(mut state): State<AppState>) -> Result<StatusCode, AppError> {
    Joke::all().delete().exec(&mut state.db).await?;
    Ok(StatusCode::OK)
}

#[instrument(skip(state))]
pub async fn get_joke(
    Path(id): Path<i64>,
    State(mut state): State<AppState>,
) -> Result<Json<Joke>, AppError> {
    let joke = Joke::get_by_id(&mut state.db, id).await?;
    Ok(Json(joke))
}

#[instrument(skip(state))]
pub async fn get_all_jokes(State(mut state): State<AppState>) -> Result<Json<Vec<Joke>>, AppError> {
    let jokes = Joke::all()
        .order_by(Joke::fields().id().asc())
        .exec(&mut state.db)
        .await?;
    Ok(Json(jokes))
}

#[instrument(skip(state))]
pub async fn get_user_jokes(
    Path(user_id): Path<i64>,
    State(mut state): State<AppState>,
) -> Result<Json<Vec<Joke>>, AppError> {
    let jokes = Joke::filter_by_user_id(user_id).exec(&mut state.db).await?;
    Ok(Json(jokes))
}

#[instrument(skip(state))]
pub async fn paginate_jokes(
    State(mut state): State<AppState>,
    Query(params): Query<PaginationParams>,
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

#[instrument(skip(state))]
pub async fn delete_joke(
    Path(id): Path<i64>,
    State(mut state): State<AppState>,
) -> Result<StatusCode, AppError> {
    Joke::delete_by_id(&mut state.db, id).await?;
    Ok(StatusCode::OK)
}
