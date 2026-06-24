use axum_everyone::{AppState, Joke, User, create_app};
use clap::Parser;
use dotenvy::dotenv;
use tokio::{net::TcpListener, signal};

use std::{
    env,
    error::Error,
    net::{Ipv4Addr, SocketAddr},
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
    tracing_subscriber::fmt().init();

    let opts = Opts::parse();

    dotenv().ok();

    let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./data.db".to_string());

    let db = toasty::Db::builder()
        .models(toasty::models!(Joke, User))
        .connect(&db_url)
        .await?;

    db.push_schema().await?;
    tracing::info!("Database schema applied");

    let state = AppState { db };

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
