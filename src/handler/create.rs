use diesel::{prelude::*, result::Error};

use crate::{
    models::{Joke, NewJoke},
    schema::jokes,
};

pub fn add(joke: NewJoke, conn: &mut diesel::SqliteConnection) -> Result<Joke, Error> {
    diesel::insert_into(jokes::table)
        .values(&joke)
        .returning(Joke::as_returning())
        .get_result(conn)
}
