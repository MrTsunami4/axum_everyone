#[utoipa::path(
    get,
    path = "/",
    tag = "Health",
    responses((status = 200, description = "Returns a greeting", body = String)),
)]
pub async fn index() -> &'static str {
    "Hello, World!"
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses((status = 200, description = "Health check passed", body = String)),
)]
pub async fn health() -> &'static str {
    "OK"
}
