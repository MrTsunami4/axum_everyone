use diesel::{prelude::*, sqlite::Sqlite};
use serde::{Deserialize, Serialize};

use crate::schema;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = schema::jokes)]
#[diesel(check_for_backend(Sqlite))]
pub struct Joke {
    pub id: i32,
    pub url: String,
}

#[derive(Insertable, Deserialize)]
#[diesel(table_name = schema::jokes)]
pub struct NewJoke {
    pub url: String,
}
