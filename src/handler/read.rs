use super::Joke;
use crate::AppState;

static QUERY: &str = r"
SELECT url FROM jokes
ORDER BY RANDOM()
LIMIT 1
";

pub async fn get_random_joke(state: AppState) -> Result<Option<Joke>, sqlx::Error> {
    let row = sqlx::query_as(QUERY).fetch_optional(&state.pool).await?;
    Ok(row)
}
