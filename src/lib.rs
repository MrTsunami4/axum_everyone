pub mod error;
pub mod handlers;
pub mod request;
pub mod router;
pub mod schemas;
pub mod state;

pub use router::create_app;

// Re-exports for convenience and toasty::models! macro discovery.
pub use request::joke_request::JokeRequest;
pub use request::user_request::UserRequest;
pub use schemas::joke::Joke;
pub use schemas::user::User;
pub use state::AppState;
