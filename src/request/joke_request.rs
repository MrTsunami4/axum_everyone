use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct JokeRequest {
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Joke content must be between 1 and 1000 characters"
    ))]
    pub content: String,
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema, IntoParams)]
pub struct PaginationParams {
    pub cursor: Option<i64>,
    pub page_size: Option<usize>,
}
