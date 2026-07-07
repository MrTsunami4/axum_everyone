use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct UserRequest {
    #[validate(length(
        min = 1,
        max = 255,
        message = "User name must be between 1 and 255 characters"
    ))]
    pub name: String,
    #[validate(email(message = "User email must be a valid email address"))]
    pub email: String,
}
