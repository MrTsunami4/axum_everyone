use serde::{Deserialize, Serialize};
use toasty::Model;
use utoipa::ToSchema;

use crate::schemas::joke::Joke;

#[derive(Debug, Clone, Serialize, Deserialize, Model, ToSchema)]
pub struct User {
    #[key]
    #[auto]
    pub id: i64,
    pub name: String,
    #[unique]
    pub email: String,
    #[has_many]
    #[serde(skip_serializing_if = "toasty::Deferred::is_unloaded", default)]
    #[schema(ignore)]
    pub jokes: toasty::Deferred<Vec<Joke>>,
    #[auto]
    #[schema(value_type = String, format = "date-time")]
    pub created_at: jiff::Timestamp,
    #[auto]
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: jiff::Timestamp,
}
