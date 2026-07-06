use axum::{body::Body, http::Request, response::Response};
use axum_everyone::{AppState, Joke, JokeRequest, SerializablePage, User, UserRequest, create_app};
use http_body_util::BodyExt;
use tower::ServiceExt;

/// Create a fresh test database with schema applied.
async fn create_test_db() -> toasty::Db {
    let db = toasty::Db::builder()
        .models(toasty::models!(crate::*))
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db.push_schema().await.unwrap();
    db
}

/// Helper to extract the JSON body from a response.
async fn json_body<T: for<'de> serde::Deserialize<'de>>(response: Response) -> T {
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

/// Create a user via the API and return it.
async fn create_user(app: axum::Router, name: &str, email: &str) -> User {
    let create_req = Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&UserRequest {
                name: name.to_string(),
                email: email.to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(create_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::CREATED);
    json_body(response).await
}

/// Create a joke for a user via the API and return it.
async fn create_joke(app: axum::Router, user_id: i64, content: &str) -> Joke {
    let create_req = Request::builder()
        .method("POST")
        .uri(format!("/users/{user_id}/jokes"))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&JokeRequest {
                content: content.to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(create_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::CREATED);
    json_body(response).await
}

#[tokio::test]
async fn test_health_check() {
    let db = create_test_db().await;
    let state = AppState { db };
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
async fn test_create_and_get_user() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let user = create_user(app.clone(), "Alice", "alice@example.com").await;
    assert_eq!(user.name, "Alice");
    assert_eq!(user.email, "alice@example.com");
    let user_id = user.id;

    let get_req = Request::builder()
        .uri(format!("/user/{user_id}"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let retrieved: User = json_body(response).await;
    assert_eq!(retrieved.id, user_id);
    assert_eq!(retrieved.name, "Alice");
}

#[tokio::test]
async fn test_create_and_get_joke() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let user = create_user(app.clone(), "Bob", "bob@example.com").await;
    let joke = create_joke(app.clone(), user.id, "Why did the chicken cross the road?").await;
    assert_eq!(joke.content, "Why did the chicken cross the road?");
    assert_eq!(joke.user_id, user.id);
    let joke_id = joke.id;

    let get_req = Request::builder()
        .uri(format!("/joke/{joke_id}"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let retrieved: Joke = json_body(response).await;
    assert_eq!(retrieved.id, joke_id);
    assert_eq!(retrieved.user_id, user.id);
}

#[tokio::test]
async fn test_user_jokes_relation() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let user = create_user(app.clone(), "Carol", "carol@example.com").await;
    create_joke(app.clone(), user.id, "Joke 1").await;
    create_joke(app.clone(), user.id, "Joke 2").await;
    create_joke(app.clone(), user.id, "Joke 3").await;

    let get_req = Request::builder()
        .uri(format!("/users/{}/jokes", user.id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let jokes: Vec<Joke> = json_body(response).await;
    assert_eq!(jokes.len(), 3);
    assert!(jokes.iter().all(|j| j.user_id == user.id));
}

#[tokio::test]
async fn test_update_joke() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let user = create_user(app.clone(), "Dave", "dave@example.com").await;
    let joke = create_joke(app.clone(), user.id, "Old joke").await;
    let joke_id = joke.id;

    let update_req = Request::builder()
        .method("PUT")
        .uri(format!("/joke/{joke_id}"))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&JokeRequest {
                content: "Updated joke".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(update_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn test_delete_joke() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let user = create_user(app.clone(), "Eve", "eve@example.com").await;
    let joke = create_joke(app.clone(), user.id, "Delete me").await;
    let joke_id = joke.id;

    let delete_req = Request::builder()
        .method("DELETE")
        .uri(format!("/joke/{joke_id}"))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let get_req = Request::builder()
        .uri(format!("/joke/{joke_id}"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_all_jokes() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let user = create_user(app.clone(), "Frank", "frank@example.com").await;
    for i in 0..3 {
        create_joke(app.clone(), user.id, &format!("Joke {i}")).await;
    }

    let get_req = Request::builder()
        .uri("/jokes")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let jokes: Vec<Joke> = json_body(response).await;
    assert_eq!(jokes.len(), 3);
}

#[tokio::test]
async fn test_validation_empty_content() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let user = create_user(app.clone(), "Grace", "grace@example.com").await;

    let create_req = Request::builder()
        .method("POST")
        .uri(format!("/users/{}/jokes", user.id))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&JokeRequest {
                content: String::new(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(create_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_delete_all_jokes() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let user = create_user(app.clone(), "Heidi", "heidi@example.com").await;
    for _ in 0..2 {
        create_joke(app.clone(), user.id, "Joke").await;
    }

    let delete_req = Request::builder()
        .method("DELETE")
        .uri("/jokes")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let get_req = Request::builder()
        .uri("/jokes")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(get_req).await.unwrap();
    let jokes: Vec<Joke> = json_body(response).await;
    assert!(jokes.is_empty());
}

#[tokio::test]
async fn test_add_joke_nonexistent_user() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let create_req = Request::builder()
        .method("POST")
        .uri("/users/9999/jokes")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&JokeRequest {
                content: "Orphan joke".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(create_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_all_users() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    create_user(app.clone(), "Ivan", "ivan@example.com").await;
    create_user(app.clone(), "Judy", "judy@example.com").await;

    let get_req = Request::builder()
        .uri("/users")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let users: Vec<User> = json_body(response).await;
    assert_eq!(users.len(), 2);
}

#[tokio::test]
async fn test_delete_user() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let user = create_user(app.clone(), "Karl", "karl@example.com").await;
    let user_id = user.id;

    let delete_req = Request::builder()
        .method("DELETE")
        .uri(format!("/user/{user_id}"))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let get_req = Request::builder()
        .uri(format!("/user/{user_id}"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_nonexistent_user() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/user/9999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_user() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let user = create_user(app.clone(), "Alice", "alice@example.com").await;
    let user_id = user.id;

    let update_req = Request::builder()
        .method("PUT")
        .uri(format!("/user/{user_id}"))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&UserRequest {
                name: "Alice Updated".to_string(),
                email: "alice.updated@example.com".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let get_req = Request::builder()
        .uri(format!("/user/{user_id}"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let retrieved: User = json_body(response).await;
    assert_eq!(retrieved.name, "Alice Updated");
    assert_eq!(retrieved.email, "alice.updated@example.com");
}

#[tokio::test]
async fn test_update_nonexistent_user() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let update_req = Request::builder()
        .method("PUT")
        .uri("/user/9999")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&UserRequest {
                name: "Ghost".to_string(),
                email: "ghost@example.com".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(update_req).await.unwrap();
    // toasty::update! succeeds even if no matching record exists
    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn test_delete_all_users() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    create_user(app.clone(), "Liam", "liam@example.com").await;
    create_user(app.clone(), "Mia", "mia@example.com").await;

    let delete_req = Request::builder()
        .method("DELETE")
        .uri("/users")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let get_req = Request::builder()
        .uri("/users")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(get_req).await.unwrap();
    let users: Vec<User> = json_body(response).await;
    assert!(users.is_empty());
}

#[tokio::test]
async fn test_validation_user_empty_name() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let create_req = Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&UserRequest {
                name: String::new(),
                email: "valid@example.com".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(create_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_validation_user_invalid_email() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let create_req = Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&UserRequest {
                name: "Noah".to_string(),
                email: "not-an-email".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(create_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_user_jokes_empty() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let user = create_user(app.clone(), "Olivia", "olivia@example.com").await;

    let get_req = Request::builder()
        .uri(format!("/users/{}/jokes", user.id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let jokes: Vec<Joke> = json_body(response).await;
    assert!(jokes.is_empty());
}

#[tokio::test]
async fn test_get_nonexistent_joke() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/joke/9999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_update_nonexistent_joke() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let update_req = Request::builder()
        .method("PUT")
        .uri("/joke/9999")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&JokeRequest {
                content: "Ghost joke".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(update_req).await.unwrap();
    // toasty::update! succeeds even if no matching record exists
    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn test_delete_nonexistent_joke() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let delete_req = Request::builder()
        .method("DELETE")
        .uri("/joke/9999")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(delete_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn test_paginate_jokes() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let user = create_user(app.clone(), "Quinn", "quinn@example.com").await;
    for i in 0..5 {
        create_joke(app.clone(), user.id, &format!("Joke {i}")).await;
    }

    let get_req = Request::builder()
        .uri("/jokes/paginate?page_size=2")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let page: SerializablePage<Joke> = json_body(response).await;
    assert_eq!(page.items.len(), 2);
    assert!(page.cursor.is_some(), "expected a cursor for the next page");

    let cursor = page.cursor.unwrap();
    let get_req = Request::builder()
        .uri(format!("/jokes/paginate?page_size=2&cursor={cursor}"))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let page: SerializablePage<Joke> = json_body(response).await;
    assert_eq!(page.items.len(), 2);
    assert!(
        page.cursor.is_some(),
        "expected a cursor for the third page"
    );

    let cursor = page.cursor.unwrap();
    let get_req = Request::builder()
        .uri(format!("/jokes/paginate?page_size=2&cursor={cursor}"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let page: SerializablePage<Joke> = json_body(response).await;
    assert_eq!(page.items.len(), 1);
    assert!(page.cursor.is_none(), "expected no cursor on the last page");
}
