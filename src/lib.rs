pub mod handler;
pub mod models;

use axum::{Router, routing::get};
use models::AppState;

/// Create the Axum router with all routes.
/// Public for integration testing.
pub fn create_app(state: AppState) -> Router {
    use tower_http::cors::CorsLayer;
    use tower_http::trace::TraceLayer;

    Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route(
            "/jokes",
            get(handler::get_all_jokes)
                .post(handler::add_joke)
                .delete(handler::delete_all_jokes),
        )
        .route(
            "/joke/{id}",
            get(handler::get_joke)
                .put(handler::update_joke)
                .delete(handler::delete_joke),
        )
        .route("/joke/random", get(handler::get_random_joke))
        .layer(CorsLayer::very_permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn index() -> &'static str {
    "Hello, World!"
}

async fn health() -> &'static str {
    "OK"
}
