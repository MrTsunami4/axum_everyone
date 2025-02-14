#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use axum::{http::StatusCode, routing::get, Router};
use clap::Parser;
use deadpool_diesel::{
    sqlite::{Manager, Pool},
    Runtime,
};
use dotenvy::dotenv;
use std::{
    error::Error,
    net::{Ipv4Addr, SocketAddr},
};
use tokio::{net::TcpListener, signal};

mod handler;
mod models;
mod schema;

#[derive(Parser)]
struct Opts {
    #[clap(long)]
    host: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let host_flag = Opts::parse().host;

    dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = Manager::new(db_url, Runtime::Tokio1);
    let pool = Pool::builder(manager).build().unwrap();

    let app = Router::new()
        .route("/", get(index))
        .route(
            "/jokes",
            get(handler::get_all_jokes)
                .post(handler::add_joke)
                .delete(handler::delete_all_joke),
        )
        .route(
            "/joke/{id}",
            get(handler::get_joke).delete(handler::delete_joke),
        )
        .with_state(pool);

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

fn internal_error<E>(_err: E) -> StatusCode
where
    E: Error,
{
    StatusCode::INTERNAL_SERVER_ERROR
}
