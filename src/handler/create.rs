use deadpool_diesel::{sqlite::Object, InteractError};
use diesel::{result::Error, RunQueryDsl, SelectableHelper};

use crate::{
    models::{Joke, NewJoke},
    schema::jokes,
};

pub async fn add(conn: Object, joke: NewJoke) -> Result<Result<Joke, Error>, InteractError> {
    conn.interact(move |conn| {
        diesel::insert_into(jokes::table)
            .values(&joke)
            .returning(Joke::as_returning())
            .get_result(conn)
    })
    .await
}
