use diesel::{prelude::*, result::Error};

use crate::schema::jokes;

pub fn remove(conn: &mut SqliteConnection) -> Result<usize, Error> {
    diesel::delete(jokes::table).execute(conn)
}

pub fn delete_joke(id: i32, conn: &mut SqliteConnection) -> Result<usize, Error> {
    diesel::delete(jokes::table.filter(jokes::id.eq(id))).execute(conn)
}
