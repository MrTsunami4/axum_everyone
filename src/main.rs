use axum_everyone::{AppState, MIGRATIONS, create_app, database};
use clap::Parser;
use dotenvy::dotenv;
use tokio::{net::TcpListener, signal};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::{
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
    dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace,toasty=debug",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let opts = Opts::parse();

    let db = database::connect(&database::database_url()?).await?;

    let report = MIGRATIONS.apply(&db).await?;
    if report.applied() > 0 {
        tracing::info!("applied {} migrations", report.applied());
    }

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
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Error setting Ctrl-C signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Error setting SIGTERM signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
