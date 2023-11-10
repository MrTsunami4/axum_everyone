#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use axum::{debug_handler, extract::State, http::StatusCode, routing::get, Json, Router};
use clap::Parser;
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

#[derive(Parser)]
struct Opts {
    #[clap(long)]
    host: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let host_flag = Opts::parse().host;

    let options = SqliteConnectOptions::new()
        .filename("data.db")
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options)
        .await
        .expect("Error connecting to database");

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
    .expect("Error creating table");

    let app = Router::new()
        .route("/", get(index))
        .route(
            "/jokes",
            get(get_random_joke).post(add_joke).delete(delete_all_joke),
        )
        .with_state(AppState { pool });

    let type_addr = if host_flag {
        Ipv4Addr::UNSPECIFIED
    } else {
        Ipv4Addr::LOCALHOST
    };
    let addr = SocketAddr::from((type_addr, 3000));
    tracing::info!("listening on http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("server failed");
}

async fn index() -> &'static str {
    "Hello, World!"
}

#[debug_handler]
async fn add_joke(state: State<AppState>, Json(payload): Json<Joke>) -> (StatusCode, Json<Joke>) {
    let result = sqlx::query(
        r#"
        INSERT INTO jokes (url)
        VALUES ($1)
        "#,
    )
    .bind(payload.url.clone())
    .execute(&state.pool)
    .await;

    match result {
        Ok(_) => (StatusCode::CREATED, Json(payload)),
        Err(result) => {
            error!("Error inserting joke: {:?}", result);
            (StatusCode::INTERNAL_SERVER_ERROR, Json(payload))
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
        Ok(Some(joke)) => (StatusCode::OK, Json(joke)),
        Ok(None) => (StatusCode::NOT_FOUND, Json(Joke { url: String::new() })),
        Err(e) => {
            error!("Error getting joke: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(Joke { url: String::new() }),
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
