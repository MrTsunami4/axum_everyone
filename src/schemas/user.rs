use serde::{Deserialize, Serialize};
use toasty::Model;

use crate::schemas::joke::Joke;

/// Represents a user who owns jokes.
#[derive(Debug, Clone, Serialize, Deserialize, Model)]
pub struct User {
    #[key]
    #[auto]
    pub id: i64,
    pub name: String,
    #[unique]
    pub email: String,
    #[has_many]
    #[serde(skip_serializing_if = "toasty::Deferred::is_unloaded", default)]
    pub jokes: toasty::Deferred<Vec<Joke>>,
    #[auto]
    pub created_at: jiff::Timestamp,
    #[auto]
    pub updated_at: jiff::Timestamp,
}
