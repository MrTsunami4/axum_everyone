use sqlx::SqlitePool;

use crate::models::Joke;

static GET_JOKE_BY_ID_QUERY: &str =
    "SELECT id, content, created_at, updated_at FROM jokes WHERE id = $1";
static GET_ALL_JOKES_QUERY: &str =
    "SELECT id, content, created_at, updated_at FROM jokes ORDER BY id LIMIT $1 OFFSET $2";
static GET_RANDOM_JOKE_QUERY: &str =
    "SELECT id, content, created_at, updated_at FROM jokes ORDER BY RANDOM() LIMIT 1";
static COUNT_JOKES_QUERY: &str = "SELECT COUNT(*) FROM jokes";

pub async fn get_joke(id: i64, pool: &SqlitePool) -> Result<Option<Joke>, sqlx::Error> {
    sqlx::query_as(GET_JOKE_BY_ID_QUERY)
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_all_jokes(
    pool: &SqlitePool,
    limit: i64,
    offset: i64,
) -> Result<Vec<Joke>, sqlx::Error> {
    sqlx::query_as(GET_ALL_JOKES_QUERY)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
}

pub async fn count_jokes(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let (count,): (i64,) = sqlx::query_as(COUNT_JOKES_QUERY).fetch_one(pool).await?;
    Ok(count)
}

pub async fn get_random_joke(pool: &SqlitePool) -> Result<Option<Joke>, sqlx::Error> {
    sqlx::query_as(GET_RANDOM_JOKE_QUERY)
        .fetch_optional(pool)
        .await
}
