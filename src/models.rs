use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Joke {
    pub id: i64,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct NewJoke {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
