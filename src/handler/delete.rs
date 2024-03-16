use crate::AppState;

static QUERY: &str = r"
DELETE FROM jokes
";

pub async fn remove(state: AppState) -> Result<u64, sqlx::Error> {
    let db = state.pool;
    let res = sqlx::query(QUERY).execute(&db).await?;
    Ok(res.rows_affected())
}
