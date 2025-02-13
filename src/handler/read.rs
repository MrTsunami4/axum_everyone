use diesel::prelude::*;
use diesel::result::Error;

use crate::models::Joke;

pub fn get_joke(id: i32, conn: &mut SqliteConnection) -> Result<Option<Joke>, Error> {
    use crate::schema::jokes::dsl::jokes;

    jokes
        .select(Joke::as_select())
        .find(id)
        .first(conn)
        .optional()
}

pub fn get_all_jokes(conn: &mut SqliteConnection) -> Result<Vec<Joke>, Error> {
    use crate::schema::jokes::dsl::jokes;

    jokes.select(Joke::as_select()).load(conn)
}
