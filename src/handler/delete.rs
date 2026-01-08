use sqlx::SqlitePool;

static DELETE_ALL_JOKES_QUERY: &str = "DELETE FROM jokes";
static DELETE_JOKE_BY_ID_QUERY: &str = "DELETE FROM jokes WHERE id = $1";

pub async fn remove(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    sqlx::query(DELETE_ALL_JOKES_QUERY)
        .execute(pool)
        .await
        .map(|result| result.rows_affected())
}

pub async fn delete_joke(id: i64, pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    sqlx::query(DELETE_JOKE_BY_ID_QUERY)
        .bind(id)
        .execute(pool)
        .await
        .map(|result| result.rows_affected())
}
