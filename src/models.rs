use serde::{Deserialize, Serialize};
use validator::Validate;

/// Application state shared across all handlers.
/// Contains the Toasty database handle.
#[derive(Clone)]
pub struct AppState {
    pub db: toasty::Db,
}

/// Represents a joke in the database.
#[derive(Debug, Clone, Serialize, Deserialize, toasty::Model)]
pub struct Joke {
    #[key]
    #[auto]
    pub id: i64,
    pub content: String,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct JokeRequest {
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Joke content must be between 1 and 1000 characters"
    ))]
    pub content: String,
}
