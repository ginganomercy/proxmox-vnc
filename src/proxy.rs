use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::{
    Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message as TungsteniteMessage};

use crate::auth::validate_jwt;
use crate::config::Config;
use crate::proxmox::get_vnc_ticket;

#[derive(Deserialize)]
pub struct VncQuery {
    token: String,
}

pub fn router(config: Arc<Config>) -> Router {
    Router::new()
        .route("/console/{node}/{vmid}", get(vnc_handler))
        .with_state(config)
}

async fn vnc_handler(
    ws: WebSocketUpgrade,
    Path((node, vmid)): Path<(String, String)>,
    Query(query): Query<VncQuery>,
    State(config): State<Arc<Config>>,
) -> impl IntoResponse {
    // 1. Validate JWT Token
    if validate_jwt(&query.token, &config.jwt_secret).is_none() {
        return axum::response::Response::builder()
            .status(401)
            .body(axum::body::Body::from("Unauthorized"))
            .unwrap();
    }

    // 2. Upgrade to WebSocket
    ws.on_upgrade(move |socket| handle_socket(socket, node, vmid, config))
}

async fn handle_socket(mut client_ws: WebSocket, node: String, vmid: String, config: Arc<Config>) {
    // 3. Request VNC Ticket from Proxmox
    let ticket_data = match get_vnc_ticket(
        &config.proxmox_url,
        &config.proxmox_token_id,
        &config.proxmox_token_secret,
        &node,
        &vmid,
    )
    .await
    {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Failed to get VNC ticket: {}", e);
            let _ = client_ws.close().await;
            return;
        }
    };

    // 4. Construct Proxmox WebSocket URL
    let wss_url = format!(
        "{}/nodes/{}/qemu/{}/vncwebsocket?port={}&vncticket={}",
        config.proxmox_url.replace("http", "ws"),
        node,
        vmid,
        ticket_data.port,
        urlencoding::encode(&ticket_data.ticket)
    );

    // 5. Connect to Proxmox WebSocket (Ignoring Certs is tricky in Rust Tungstenite, need to configure TLS)
    // For simplicity, we use native-tls with danger_accept_invalid_certs if required, but tokio-tungstenite needs a custom connector.
    // Let's use the default connect_async first.
    let (proxmox_ws, _) = match connect_async(&wss_url).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Failed to connect to Proxmox WS: {}", e);
            let _ = client_ws.close().await;
            return;
        }
    };

    println!("VNC Proxy established for VM {} on node {}", vmid, node);

    // 6. Bidirectional Piping
    let (mut client_tx, mut client_rx) = client_ws.split();
    let (mut proxmox_tx, mut proxmox_rx) = proxmox_ws.split();

    let client_to_proxmox = tokio::spawn(async move {
        while let Some(Ok(msg)) = client_rx.next().await {
            let t_msg = match msg {
                Message::Text(t) => TungsteniteMessage::Text(t.to_string().into()),
                Message::Binary(b) => TungsteniteMessage::Binary(b.into()),
                Message::Ping(p) => TungsteniteMessage::Ping(p.into()),
                Message::Pong(p) => TungsteniteMessage::Pong(p.into()),
                Message::Close(_) => break,
            };
            if proxmox_tx.send(t_msg).await.is_err() {
                break;
            }
        }
    });

    let proxmox_to_client = tokio::spawn(async move {
        while let Some(Ok(msg)) = proxmox_rx.next().await {
            let axum_msg = match msg {
                TungsteniteMessage::Text(t) => Message::Text(t.to_string().into()),
                TungsteniteMessage::Binary(b) => Message::Binary(b.into()),
                TungsteniteMessage::Ping(p) => Message::Ping(p.into()),
                TungsteniteMessage::Pong(p) => Message::Pong(p.into()),
                TungsteniteMessage::Close(_) => break,
                TungsteniteMessage::Frame(_) => continue,
            };
            if client_tx.send(axum_msg).await.is_err() {
                break;
            }
        }
    });

    tokio::select! {
        _ = client_to_proxmox => (),
        _ = proxmox_to_client => (),
    }

    println!("VNC Proxy closed for VM {}", vmid);
}
