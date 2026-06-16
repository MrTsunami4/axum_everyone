use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct JokeRequest {
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Joke content must be between 1 and 1000 characters"
    ))]
    pub content: String,
}
