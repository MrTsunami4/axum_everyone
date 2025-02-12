use deadpool_diesel::{sqlite::Object, InteractError};
use diesel::prelude::*;
use diesel::result::Error;

use crate::models::Joke;

pub async fn get_joke(id: i32, conn: Object) -> Result<Result<Option<Joke>, Error>, InteractError> {
    use crate::schema::jokes::dsl::jokes;

    conn.interact(move |conn| {
        jokes
            .select(Joke::as_select())
            .find(id)
            .first(conn)
            .optional()
    })
    .await
}

pub(crate) async fn get_all_jokes(conn: Object) -> Result<Result<Vec<Joke>, Error>, InteractError> {
    use crate::schema::jokes::dsl::jokes;

    conn.interact(move |conn| jokes.select(Joke::as_select()).load(conn))
        .await
}
