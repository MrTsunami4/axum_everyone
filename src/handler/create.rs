use crate::{AppState, Joke};

static QUERY: &str = r"
INSERT INTO jokes (url)
VALUES ($1)
";

pub async fn add(state: AppState, joke: Joke) -> Result<Joke, sqlx::Error> {
    let db = state.pool;
    let result = sqlx::query_as(QUERY).bind(joke.url).fetch_one(&db).await?;
    Ok(result)
}
