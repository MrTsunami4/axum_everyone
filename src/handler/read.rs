use sqlx::SqlitePool;

use crate::models::Joke;

static GET_JOKE_BY_ID_QUERY: &str = "SELECT id, content FROM jokes WHERE id = $1";
static GET_ALL_JOKES_QUERY: &str = "SELECT id, content FROM jokes";
static GET_RANDOM_JOKE_QUERY: &str = "SELECT id, content FROM jokes ORDER BY RANDOM() LIMIT 1";

pub async fn get_joke(id: i64, pool: &SqlitePool) -> Result<Option<Joke>, sqlx::Error> {
    sqlx::query_as(GET_JOKE_BY_ID_QUERY)
        .bind(id)
        .fetch_optional(pool)
        .await
}

pub async fn get_all_jokes(pool: &SqlitePool) -> Result<Vec<Joke>, sqlx::Error> {
    sqlx::query_as(GET_ALL_JOKES_QUERY).fetch_all(pool).await
}

pub async fn get_random_joke(pool: &SqlitePool) -> Result<Option<Joke>, sqlx::Error> {
    sqlx::query_as(GET_RANDOM_JOKE_QUERY)
        .fetch_optional(pool)
        .await
}
