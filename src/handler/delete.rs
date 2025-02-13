use diesel::{prelude::*, result::Error};

use crate::schema::jokes;

pub fn remove(conn: &mut SqliteConnection) -> Result<usize, Error> {
    diesel::delete(jokes::table).execute(conn)
}
