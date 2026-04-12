use axum::{body::Body, http::Request, response::Response};
use axum_everyone::{
    create_app,
    models::{AppState, CreateJokeRequest, Joke, PaginatedJokesResponse, UpdateJokeRequest},
};
use http_body_util::BodyExt;
use sqlx::SqlitePool;
use tower::ServiceExt;

/// Create a fresh test database pool with migrations applied.
async fn create_test_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

/// Helper to extract the JSON body from a response.
async fn json_body<T: for<'de> serde::Deserialize<'de>>(response: Response) -> T {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn test_health_check() {
    let pool = create_test_pool().await;
    let state = AppState { db: pool };
    let app = create_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(std::str::from_utf8(&bytes).unwrap(), "OK");
}

#[tokio::test]
async fn test_create_and_get_joke() {
    let pool = create_test_pool().await;
    let state = AppState { db: pool };
    let app = create_app(state);

    // Create a joke
    let create_req = Request::builder()
        .method("POST")
        .uri("/jokes")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&CreateJokeRequest {
                content: "Why did the chicken cross the road?".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::CREATED);

    let joke: Joke = json_body(response).await;
    assert_eq!(joke.content, "Why did the chicken cross the road?");
    let joke_id = joke.id;

    // Get the joke by ID
    let get_req = Request::builder()
        .uri(format!("/joke/{joke_id}"))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let retrieved: Joke = json_body(response).await;
    assert_eq!(retrieved.id, joke_id);
}

#[tokio::test]
async fn test_update_joke() {
    let pool = create_test_pool().await;
    let state = AppState { db: pool };
    let app = create_app(state);

    // Create a joke first
    let create_req = Request::builder()
        .method("POST")
        .uri("/jokes")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&CreateJokeRequest {
                content: "Old joke".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_req).await.unwrap();
    let joke: Joke = json_body(response).await;
    let joke_id = joke.id;

    // Update the joke
    let update_req = Request::builder()
        .method("PUT")
        .uri(format!("/joke/{joke_id}"))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&UpdateJokeRequest {
                content: "Updated joke".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let updated: Joke = json_body(response).await;
    assert_eq!(updated.content, "Updated joke");
}

#[tokio::test]
async fn test_delete_joke() {
    let pool = create_test_pool().await;
    let state = AppState { db: pool };
    let app = create_app(state);

    // Create a joke first
    let create_req = Request::builder()
        .method("POST")
        .uri("/jokes")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&CreateJokeRequest {
                content: "Delete me".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_req).await.unwrap();
    let joke: Joke = json_body(response).await;
    let joke_id = joke.id;

    // Delete the joke
    let delete_req = Request::builder()
        .method("DELETE")
        .uri(format!("/joke/{joke_id}"))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    // Try to get the deleted joke - should be 404
    let get_req = Request::builder()
        .uri(format!("/joke/{joke_id}"))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_all_jokes_with_pagination() {
    let pool = create_test_pool().await;
    let state = AppState { db: pool };
    let app = create_app(state);

    // Create 3 jokes
    for i in 0..3 {
        let create_req = Request::builder()
            .method("POST")
            .uri("/jokes")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&CreateJokeRequest {
                    content: format!("Joke {i}"),
                })
                .unwrap(),
            ))
            .unwrap();

        let _response = app.clone().oneshot(create_req).await.unwrap();
    }

    // Get all jokes
    let get_req = Request::builder()
        .uri("/jokes")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let paginated: PaginatedJokesResponse = json_body(response).await;
    assert_eq!(paginated.total, 3);
    assert_eq!(paginated.jokes.len(), 3);
    assert_eq!(paginated.page, 1);
    assert_eq!(paginated.per_page, 20);
}

#[tokio::test]
async fn test_validation_empty_content() {
    let pool = create_test_pool().await;
    let state = AppState { db: pool };
    let app = create_app(state);

    let create_req = Request::builder()
        .method("POST")
        .uri("/jokes")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&CreateJokeRequest {
                content: "".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(
        response.status(),
        axum::http::StatusCode::UNPROCESSABLE_ENTITY
    );
}

#[tokio::test]
async fn test_random_joke() {
    let pool = create_test_pool().await;
    let state = AppState { db: pool };
    let app = create_app(state);

    // Create a joke first
    let create_req = Request::builder()
        .method("POST")
        .uri("/jokes")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&CreateJokeRequest {
                content: "Random joke".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let _response = app.clone().oneshot(create_req).await.unwrap();

    // Get random joke
    let get_req = Request::builder()
        .uri("/joke/random")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let joke: Joke = json_body(response).await;
    assert_eq!(joke.content, "Random joke");
}

#[tokio::test]
async fn test_delete_all_jokes() {
    let pool = create_test_pool().await;
    let state = AppState { db: pool };
    let app = create_app(state);

    // Create 2 jokes
    for _ in 0..2 {
        let create_req = Request::builder()
            .method("POST")
            .uri("/jokes")
            .header("content-type", "application/json")
            .body(Body::from(
                serde_json::to_string(&CreateJokeRequest {
                    content: "Joke".to_string(),
                })
                .unwrap(),
            ))
            .unwrap();

        let _response = app.clone().oneshot(create_req).await.unwrap();
    }

    // Delete all jokes
    let delete_req = Request::builder()
        .method("DELETE")
        .uri("/jokes")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    // Verify no jokes left
    let get_req = Request::builder()
        .uri("/jokes")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(get_req).await.unwrap();
    let paginated: PaginatedJokesResponse = json_body(response).await;
    assert_eq!(paginated.total, 0);
}
