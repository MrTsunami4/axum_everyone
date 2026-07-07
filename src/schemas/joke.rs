use serde::{Deserialize, Serialize};
use toasty::Model;
use utoipa::ToSchema;

use crate::schemas::user::User;

#[derive(Debug, Clone, Serialize, Deserialize, Model, ToSchema)]
pub struct Joke {
    #[key]
    #[auto]
    pub id: i64,
    pub content: String,
    #[index]
    pub user_id: i64,
    #[belongs_to(key = user_id, references = id)]
    #[serde(skip_serializing_if = "toasty::Deferred::is_unloaded", default)]
    #[schema(ignore)]
    pub user: toasty::Deferred<User>,
    #[auto]
    #[schema(value_type = String, format = "date-time")]
    pub created_at: jiff::Timestamp,
    #[auto]
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: jiff::Timestamp,
}
