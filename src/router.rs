use axum::{Router, routing::get};

use crate::handlers;
use crate::state::AppState;

/// Create the Axum router with all routes.
/// Public for integration testing.
pub fn create_app(state: AppState) -> Router {
    use tower_http::cors::CorsLayer;
    use tower_http::trace::TraceLayer;

    Router::new()
        .route("/", get(handlers::health::index))
        .route("/health", get(handlers::health::health))
        .route(
            "/jokes",
            get(handlers::jokes::get_all_jokes)
                .post(handlers::jokes::add_joke)
                .delete(handlers::jokes::delete_all_jokes),
        )
        .route(
            "/joke/{id}",
            get(handlers::jokes::get_joke)
                .put(handlers::jokes::update_joke)
                .delete(handlers::jokes::delete_joke),
        )
        .layer(CorsLayer::very_permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
