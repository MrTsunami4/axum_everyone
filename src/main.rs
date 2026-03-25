use axum::{Router, routing::get};
use clap::Parser;
use dotenvy::dotenv;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tokio::{net::TcpListener, signal};

use std::{
    error::Error,
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};

mod handler;
mod models;

#[derive(Parser)]
struct Opts {
    /// Bind to all interfaces instead of localhost
    #[clap(long)]
    host: bool,
    /// Port to listen on
    #[clap(long, default_value = "3000")]
    port: u16,
}

static INITIALIZE_DB_QUERY: &str = "CREATE TABLE IF NOT EXISTS jokes (
    id INTEGER PRIMARY KEY,
    content TEXT NOT NULL
)";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let opts = Opts::parse();

    dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(20)
        .connect_with(options)
        .await?;

    sqlx::query(INITIALIZE_DB_QUERY).execute(&pool).await?;

    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route(
            "/jokes",
            get(handler::get_all_jokes)
                .post(handler::add_joke)
                .delete(handler::delete_all_joke),
        )
        .route("/joke/random", get(handler::get_random_joke))
        .route(
            "/joke/{id}",
            get(handler::get_joke).delete(handler::delete_joke),
        )
        .with_state(pool);

    let bind_addr = if opts.host {
        Ipv4Addr::UNSPECIFIED
    } else {
        Ipv4Addr::LOCALHOST
    };
    let addr = SocketAddr::from((bind_addr, opts.port));
    let tcp = TcpListener::bind(addr).await?;
    tracing::info!("listening on http://{}", addr);
    axum::serve(tcp, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    signal::ctrl_c()
        .await
        .expect("Error setting Ctrl-C signal handler");
}

async fn index() -> &'static str {
    "Hello, World!"
}

async fn health() -> &'static str {
    "OK"
}
