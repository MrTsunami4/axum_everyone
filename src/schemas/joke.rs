use serde::{Deserialize, Serialize};
use toasty::Model;

/// Represents a joke in the database.
#[derive(Debug, Clone, Serialize, Deserialize, Model)]
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
