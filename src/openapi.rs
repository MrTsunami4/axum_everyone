use utoipa::OpenApi;

use crate::SerializablePage;
use crate::request::joke_request::{JokeRequest, PaginationParams};
use crate::request::user_request::UserRequest;
use crate::schemas::joke::Joke;
use crate::schemas::user::User;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Axum Everyone API",
        description = "A joke-sharing API built with Axum and Toasty",
        version = "0.1.0"
    ),
    components(
        schemas(
            User,
            Joke,
            UserRequest,
            JokeRequest,
            PaginationParams,
            SerializablePage<Joke>,
        )
    ),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Users", description = "User management endpoints"),
        (name = "Jokes", description = "Joke management endpoints"),
    ),
)]
pub struct ApiDoc;
