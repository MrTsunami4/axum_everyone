use sqlx::SqlitePool;

use crate::models::{Joke, NewJoke};

static INSERT_JOKE_QUERY: &str = "INSERT INTO jokes (content) VALUES ($1) RETURNING id, content";

pub async fn add(joke: &NewJoke, pool: &SqlitePool) -> Result<Joke, sqlx::Error> {
    let result = sqlx::query_as(INSERT_JOKE_QUERY)
        .bind(&joke.content)
        .fetch_one(pool)
        .await?;

    Ok(result)
}
