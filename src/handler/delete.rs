use deadpool_diesel::{sqlite::Object, InteractError};
use diesel::{result::Error, RunQueryDsl};

use crate::schema::jokes;

// static QUERY: &str = r"
// DELETE FROM jokes
// ";

pub async fn remove(conn: Object) -> Result<Result<usize, Error>, InteractError> {
    conn.interact(|conn| diesel::delete(jokes::table).execute(conn))
        .await
}
