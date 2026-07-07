use axum::Router;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers;
use crate::openapi::ApiDoc;
use crate::state::AppState;

/// Create the Axum router with all routes.
/// Public for integration testing.
pub fn create_app(state: AppState) -> Router {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(utoipa_axum::routes!(handlers::health::index))
        .routes(utoipa_axum::routes!(handlers::health::health))
        .routes(utoipa_axum::routes!(
            handlers::users::get_all_users,
            handlers::users::add_user,
            handlers::users::delete_all_users,
        ))
        .routes(utoipa_axum::routes!(
            handlers::users::get_user,
            handlers::users::update_user,
            handlers::users::delete_user,
        ))
        .routes(utoipa_axum::routes!(
            handlers::jokes::get_user_jokes,
            handlers::jokes::add_joke,
        ))
        .routes(utoipa_axum::routes!(
            handlers::jokes::get_all_jokes,
            handlers::jokes::delete_all_jokes,
        ))
        .routes(utoipa_axum::routes!(handlers::jokes::paginate_jokes))
        .routes(utoipa_axum::routes!(
            handlers::jokes::get_joke,
            handlers::jokes::update_joke,
            handlers::jokes::delete_joke,
        ))
        .layer(CorsLayer::very_permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
        .split_for_parts();

    router.merge(SwaggerUi::new("/api").url("/api-docs/openapi.json", api))
}
