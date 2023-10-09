use axum::{debug_handler, extract::State, http::StatusCode, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteConnectOptions, FromRow, SqlitePool};
use std::net::{Ipv4Addr, SocketAddr};
use tracing::error;

#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
}

#[derive(Serialize, Deserialize, Debug, FromRow)]
struct Joke {
    url: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let options = SqliteConnectOptions::new()
        .filename("data.db")
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options).await.unwrap();

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS jokes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let app = Router::new()
        .route("/", get(index))
        .route(
            "/jokes",
            get(get_random_joke).post(add_joke).delete(delete_all_joke),
        )
        .with_state(AppState { pool });

    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 3000));
    tracing::info!("listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index() -> &'static str {
    "Hello, World!"
}

#[debug_handler]
async fn add_joke(state: State<AppState>, Json(payload): Json<Joke>) -> (StatusCode, Json<Joke>) {
    let joke = Joke { url: payload.url };

    let result = sqlx::query(
        r#"
        INSERT INTO jokes (url)
        VALUES ($1)
        "#,
    )
    .bind(joke.url.clone())
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => (StatusCode::CREATED, Json(joke)),
        Err(result) => {
            error!("Error inserting joke: {:?}", result);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(joke))
        }
    }
}

#[debug_handler]
async fn get_random_joke(state: State<AppState>) -> (StatusCode, Json<Joke>) {
    let row = sqlx::query_as::<_, Joke>(
        r#"
        SELECT url FROM jokes
        ORDER BY RANDOM()
        LIMIT 1
        "#,
    )
    .fetch_optional(&state.pool)
    .await;

    match row {
        Ok(opt) => match opt {
            Some(joke) => (StatusCode::OK, Json(joke)),
            None => (
                StatusCode::NOT_FOUND,
                Json(Joke {
                    url: "".to_string(),
                }),
            ),
        },
        Err(e) => {
            error!("Error getting joke: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Joke {
                    url: "".to_string(),
                }),
            )
        }
    }
}

#[debug_handler]
async fn delete_all_joke(state: State<AppState>) -> StatusCode {
    let result = sqlx::query(
        r#"
        DELETE FROM jokes
        "#,
    )
    .execute(&state.pool)
    .await;

    match result {
        Ok(qr) => {
            if qr.rows_affected() > 0 {
                StatusCode::OK
            } else {
                StatusCode::NOT_MODIFIED
            }
        }
        Err(result) => {
            error!("Error deleting jokes: {:?}", result);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
