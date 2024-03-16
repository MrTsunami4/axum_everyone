#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use axum::{routing::get, Router};
use clap::Parser;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteConnectOptions, FromRow, SqlitePool};
use std::net::{Ipv4Addr, SocketAddr};
use tokio::{net::TcpListener, signal};

mod handler;

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

    init_table(&pool).await;

    let app = Router::new()
        .route("/", get(index))
        .route(
            "/jokes",
            get(handler::get_random_joke)
                .post(handler::add_joke)
                .delete(handler::delete_all_joke),
        )
        .with_state(AppState { pool });

    let type_addr = if host_flag {
        Ipv4Addr::UNSPECIFIED
    } else {
        Ipv4Addr::LOCALHOST
    };
    let addr = SocketAddr::from((type_addr, 3000));
    let tcp = TcpListener::bind(addr).await.unwrap();
    tracing::info!("listening on http://{}", addr);
    axum::serve(tcp, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    signal::ctrl_c()
        .await
        .expect("Error setting Ctrl-C signal handler");
}

async fn index() -> &'static str {
    "Hello, World!"
}

async fn init_table(pool: &SqlitePool) {
    sqlx::query(
        r"
        CREATE TABLE IF NOT EXISTS jokes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            url TEXT NOT NULL
        )
        ",
    )
    .execute(pool)
    .await
    .expect("Error creating table");
}
