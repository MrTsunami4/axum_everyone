pub mod error;
pub mod handlers;
pub mod openapi;
pub mod request;
pub mod router;
pub mod schemas;
pub mod state;

pub use router::create_app;

// Re-exports for convenience and toasty::models! macro discovery.
pub use request::{joke_request::JokeRequest, user_request::UserRequest};
pub use schemas::{joke::Joke, user::User};
pub use state::AppState;
use toasty::stmt::{Page, Value};
use utoipa::ToSchema;

#[derive(serde::Serialize, serde::Deserialize, ToSchema)]
pub struct SerializablePage<T> {
    pub items: Vec<T>,
    pub cursor: Option<i64>,
}

fn cursor_to_i64(cursor: Value) -> i64 {
    cursor
        .as_record()
        .expect("cursor is not a record")
        .first()
        .expect("cursor record is empty")
        .to_i64()
        .expect("cursor record field is not i64")
}

impl<T> From<Page<T>> for SerializablePage<T> {
    fn from(page: Page<T>) -> Self {
        Self {
            items: page.items,
            cursor: page.next_cursor.map(cursor_to_i64),
        }
    }
}
