use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

/// Application state shared across all handlers.
/// Contains the database connection pool.
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
}

/// Represents a joke in the database.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Joke {
    pub id: i64,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Request body for creating a new joke.
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateJokeRequest {
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Joke content must be between 1 and 1000 characters"
    ))]
    pub content: String,
}

/// Request body for updating an existing joke.
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateJokeRequest {
    #[validate(length(
        min = 1,
        max = 1000,
        message = "Joke content must be between 1 and 1000 characters"
    ))]
    pub content: String,
}

/// Response body for paginated joke lists.
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedJokesResponse {
    pub jokes: Vec<Joke>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

/// Standard error response.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Query parameters for listing jokes with pagination.
#[derive(Debug, Deserialize)]
pub struct JokeQueryParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

impl JokeQueryParams {
    #[must_use]
    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    #[must_use]
    pub fn per_page(&self) -> i64 {
        self.per_page.unwrap_or(20).clamp(1, 100)
    }

    #[must_use]
    pub fn offset(&self) -> i64 {
        (self.page() - 1) * self.per_page()
    }
}
