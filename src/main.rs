mod core;
mod handlers;

use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Arc::new(core::config::Config::load());

    let app = handlers::vnc::router(config.clone());

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = TcpListener::bind(&addr).await.unwrap();

    println!("Starting CBT VNC Proxy (Rust) on ws://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
