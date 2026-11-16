use axum::{body::Body, http::Request, response::Response};
use axum_everyone::{
    AppState, ErrorResponse, Joke, JokeRequest, MIGRATIONS, SerializablePage, User, UserRequest,
    create_app,
};
use http_body_util::BodyExt;
use tower::ServiceExt;

/// Create a fresh test database with migrations applied.
async fn create_test_db() -> toasty::Db {
    let db = axum_everyone::database::connect("sqlite::memory:")
        .await
        .unwrap();
    MIGRATIONS.apply(&db).await.unwrap();
    db
}

#[tokio::test]
async fn test_invalid_path_parameters_return_json() {
    let app = create_app(AppState {
        db: create_test_db().await,
    });
    for (method, uri) in [
        ("GET", "/users/abc"),
        ("PUT", "/users/abc"),
        ("DELETE", "/users/abc"),
        ("GET", "/jokes/9223372036854775808"),
        ("PUT", "/jokes/abc"),
        ("DELETE", "/jokes/abc"),
        ("GET", "/users/abc/jokes"),
        ("POST", "/users/abc/jokes"),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            axum::http::StatusCode::BAD_REQUEST,
            "{method} {uri}"
        );
        let error: ErrorResponse = json_body(response).await;
        assert!(!error.message.is_empty());
    }
}

#[tokio::test]
async fn test_delete_user_cascades_and_preserves_other_users() {
    let app = create_app(AppState {
        db: create_test_db().await,
    });
    let alice = create_user(app.clone(), "Alice", "alice@example.com").await;
    let bob = create_user(app.clone(), "Bob", "bob@example.com").await;
    let deleted = create_joke(app.clone(), alice.id, "Alice's joke").await;
    let retained = create_joke(app.clone(), bob.id, "Bob's joke").await;
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/users/{}", alice.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NO_CONTENT);
    for (uri, status) in [
        (
            format!("/jokes/{}", deleted.id),
            axum::http::StatusCode::NOT_FOUND,
        ),
        (
            format!("/jokes/{}", retained.id),
            axum::http::StatusCode::OK,
        ),
        (format!("/users/{}", bob.id), axum::http::StatusCode::OK),
    ] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), status);
    }
}

#[tokio::test]
async fn test_delete_all_users_with_jokes_is_repeatable() {
    let app = create_app(AppState {
        db: create_test_db().await,
    });
    for (name, email) in [("Alice", "alice@example.com"), ("Bob", "bob@example.com")] {
        let user = create_user(app.clone(), name, email).await;
        create_joke(app.clone(), user.id, "A joke").await;
    }
    for _ in 0..2 {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/users")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::NO_CONTENT);
    }
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/jokes")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let page: SerializablePage<Joke> = json_body(response).await;
    assert!(page.items.is_empty());
    assert!(page.cursor.is_none());
    let response = app
        .oneshot(
            Request::builder()
                .uri("/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(json_body::<Vec<User>>(response).await.is_empty());
}

#[tokio::test]
async fn test_openapi_matches_routes_and_path_parameters() {
    let app = create_app(AppState {
        db: create_test_db().await,
    });
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api-docs/openapi.json")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let doc: serde_json::Value = json_body(response).await;
    for (path, methods) in [
        ("/", vec!["get"]),
        ("/health", vec!["get"]),
        ("/users", vec!["get", "post", "delete"]),
        ("/users/{id}", vec!["get", "put", "delete"]),
        ("/users/{user_id}/jokes", vec!["get", "post"]),
        ("/jokes", vec!["get", "delete"]),
        ("/jokes/{id}", vec!["get", "put", "delete"]),
    ] {
        for method in methods {
            let operation = &doc["paths"][path][method];
            assert!(operation.is_object(), "missing {method} {path}");
            if path.contains('{') {
                assert!(
                    operation["parameters"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|param| param["in"] == "path" && param["schema"]["type"] == "integer")
                );
            }
        }
    }
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
        .uri(format!("/users/{user_id}"))
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
        .uri(format!("/jokes/{joke_id}"))
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
        .uri(format!("/jokes/{joke_id}"))
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
        .uri(format!("/jokes/{joke_id}"))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NO_CONTENT);

    let get_req = Request::builder()
        .uri(format!("/jokes/{joke_id}"))
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

    let page: SerializablePage<Joke> = json_body(response).await;
    assert_eq!(page.items.len(), 3);
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
    assert_eq!(response.status(), axum::http::StatusCode::NO_CONTENT);

    let get_req = Request::builder()
        .uri("/jokes")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(get_req).await.unwrap();
    let page: SerializablePage<Joke> = json_body(response).await;
    assert!(page.items.is_empty());
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
        .uri(format!("/users/{user_id}"))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(delete_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NO_CONTENT);

    let get_req = Request::builder()
        .uri(format!("/users/{user_id}"))
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
                .uri("/users/9999")
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
        .uri(format!("/users/{user_id}"))
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
        .uri(format!("/users/{user_id}"))
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
        .uri("/users/9999")
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
    // Missing user: the endpoint should 404 rather than silently succeed.
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
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
    assert_eq!(response.status(), axum::http::StatusCode::NO_CONTENT);

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
                .uri("/jokes/9999")
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
        .uri("/jokes/9999")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&JokeRequest {
                content: "Ghost joke".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(update_req).await.unwrap();
    // Missing joke: the endpoint should 404 rather than silently succeed.
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_nonexistent_joke() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let delete_req = Request::builder()
        .method("DELETE")
        .uri("/jokes/9999")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(delete_req).await.unwrap();
    // Missing joke: the endpoint should 404 rather than silently succeed.
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_paginate_jokes() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let user = create_user(app.clone(), "Quinn", "quinn@example.com").await;
    let mut expected_ids = Vec::new();
    for i in 0..5 {
        expected_ids.push(
            create_joke(app.clone(), user.id, &format!("Joke {i}"))
                .await
                .id,
        );
    }

    let get_req = Request::builder()
        .uri("/jokes?page_size=2")
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let page: SerializablePage<Joke> = json_body(response).await;
    assert_eq!(page.items.len(), 2);
    assert_eq!(
        page.items.iter().map(|joke| joke.id).collect::<Vec<_>>(),
        expected_ids[..2]
    );
    assert!(page.cursor.is_some(), "expected a cursor for the next page");

    let cursor = page.cursor.unwrap();
    let get_req = Request::builder()
        .uri(format!("/jokes?page_size=2&cursor={cursor}"))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let page: SerializablePage<Joke> = json_body(response).await;
    assert_eq!(page.items.len(), 2);
    assert_eq!(
        page.items.iter().map(|joke| joke.id).collect::<Vec<_>>(),
        expected_ids[2..4]
    );
    assert!(
        page.cursor.is_some(),
        "expected a cursor for the third page"
    );

    let cursor = page.cursor.unwrap();
    let get_req = Request::builder()
        .uri(format!("/jokes?page_size=2&cursor={cursor}"))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    let page: SerializablePage<Joke> = json_body(response).await;
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].id, expected_ids[4]);
    assert!(page.cursor.is_none(), "expected no cursor on the last page");
}

#[tokio::test]
async fn test_delete_nonexistent_user() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let delete_req = Request::builder()
        .method("DELETE")
        .uri("/users/9999")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(delete_req).await.unwrap();
    // Missing user: the endpoint should 404 rather than silently succeed.
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_user_jokes_nonexistent_user() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let get_req = Request::builder()
        .uri("/users/424242/jokes")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(get_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_add_duplicate_user_conflict() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    create_user(app.clone(), "Alice", "alice@example.com").await;

    let create_req = Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&UserRequest {
                name: "Alice Clone".to_string(),
                email: "alice@example.com".to_string(),
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.oneshot(create_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::CONFLICT);
    let err: ErrorResponse = json_body(response).await;
    assert!(!err.message.is_empty());
}

#[tokio::test]
async fn test_update_user_duplicate_email_conflict() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    let alice = create_user(app.clone(), "Alice", "alice@example.com").await;
    let bob = create_user(app.clone(), "Bob", "bob@example.com").await;

    let update_req = Request::builder()
        .method("PUT")
        .uri(format!("/users/{}", bob.id))
        .header("content-type", "application/json")
        .body(Body::from(
            serde_json::to_string(&UserRequest {
                name: "Bob".to_string(),
                email: alice.email,
            })
            .unwrap(),
        ))
        .unwrap();

    let response = app.clone().oneshot(update_req).await.unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::CONFLICT);
    let err: ErrorResponse = json_body(response).await;
    assert!(!err.message.is_empty());

    // A rejected update must preserve the record and release its transaction.
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/users/{}", bob.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let unchanged: User = json_body(response).await;
    assert_eq!(unchanged.name, "Bob");
    assert_eq!(unchanged.email, "bob@example.com");
    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri(format!("/users/{}", bob.id))
                .header("content-type", "application/json")
                .body(Body::from(r#"{"name":"Robert","email":"bob@example.com"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), axum::http::StatusCode::OK);
}

#[tokio::test]
async fn test_paginate_invalid_params_rejected() {
    let db = create_test_db().await;
    let state = AppState { db };
    let app = create_app(state);

    // page_size=0 used to panic inside toasty; it must be rejected as a bad
    // request instead.
    for uri in [
        "/jokes?page_size=0",
        "/jokes?page_size=101",
        "/jokes?page_size=abc",
        "/jokes?cursor=abc",
    ] {
        let response = app
            .clone()
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(
            response.status(),
            axum::http::StatusCode::BAD_REQUEST,
            "expected 400 for {uri}"
        );
        // Errors must use the standardized JSON envelope, not plain text.
        let err: ErrorResponse = json_body(response).await;
        assert!(!err.message.is_empty(), "empty message for {uri}");
    }
}
