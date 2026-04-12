use axum_everyone::{create_app, models::AppState};
use clap::Parser;
use dotenvy::dotenv;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use tokio::{net::TcpListener, signal};

use std::{
    error::Error,
    net::{Ipv4Addr, SocketAddr},
    str::FromStr,
};

#[derive(Parser)]
struct Opts {
    /// Bind to all interfaces instead of localhost
    #[clap(long)]
    host: bool,
    /// Port to listen on
    #[clap(long, default_value = "3000")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    let opts = Opts::parse();

    dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let options = SqliteConnectOptions::from_str(&db_url)?.create_if_missing(true);
    let pool = SqlitePoolOptions::new()
        .max_connections(20)
        .connect_with(options)
        .await?;

    // Run migrations on startup
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database migrations applied");

    let state = AppState { db: pool };

    let app = create_app(state);

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
