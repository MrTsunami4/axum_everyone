use sqlx::SqlitePool;

use crate::models::{Joke, UpdateJokeRequest};

static UPDATE_JOKE_QUERY: &str =
    "UPDATE jokes SET content = $1, updated_at = CURRENT_TIMESTAMP WHERE id = $2 RETURNING id, content, created_at, updated_at";

pub async fn update(
    id: i64,
    joke: &UpdateJokeRequest,
    pool: &SqlitePool,
) -> Result<Option<Joke>, sqlx::Error> {
    sqlx::query_as(UPDATE_JOKE_QUERY)
        .bind(&joke.content)
        .bind(id)
        .fetch_optional(pool)
        .await
}
