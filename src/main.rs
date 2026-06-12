mod auth;
mod config;
mod proxmox;
mod proxy;

use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Arc::new(config::Config::load());

    let app = proxy::router(config.clone());

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await.unwrap();

    println!("Starting CBT VNC Proxy (Rust) on ws://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
