use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::{
    Router,
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
};
use futures_util::{SinkExt, StreamExt};
use native_tls::TlsConnector;
use std::borrow::Cow;
use std::sync::Arc;
use tokio_tungstenite::{Connector, tungstenite::protocol::Message as TungsteniteMessage};

use crate::core::auth::validate_jwt;
use crate::core::config::Config;

#[derive(serde::Deserialize)]
pub struct VncQuery {
    pub port: String,
    pub vncticket: String,
}

pub fn router(config: Arc<Config>) -> Router {
    Router::new()
        .route("/console/{node}/{vmid}", get(vnc_handler))
        .with_state(config)
}

async fn vnc_handler(
    ws: WebSocketUpgrade,
    headers: axum::http::HeaderMap,
    Path((node, vmid)): Path<(String, String)>,
    axum::extract::Query(query): axum::extract::Query<VncQuery>,
    State(config): State<Arc<Config>>,
) -> impl IntoResponse {
    // 0. Prevent Path Traversal
    if !node.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        || !vmid.chars().all(|c| c.is_ascii_digit())
    {
        return axum::response::Response::builder()
            .status(400)
            .body(axum::body::Body::from(
                "Invalid parameter format (potential path traversal detected)",
            ))
            .unwrap();
    }

    // 1. Extract JWT Token from Sec-WebSocket-Protocol header or Query
    let mut token = String::new();

    if let Some(protocol_str) = headers
        .get("Sec-WebSocket-Protocol")
        .and_then(|h| h.to_str().ok())
    {
        let parts: Vec<&str> = protocol_str.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 && parts[0] == "jwt" {
            token = parts[1].to_string();
        }
    }

    // 2. Validate JWT Token
    if token.is_empty() || validate_jwt(&token, &config.jwt_secret).is_none() {
        return axum::response::Response::builder()
            .status(401)
            .body(axum::body::Body::from("Unauthorized"))
            .unwrap();
    }

    // 3. Upgrade to WebSocket, responding with the accepted subprotocol
    ws.protocols([Cow::Borrowed("jwt"), Cow::Owned(token)])
        .on_upgrade(move |socket| handle_socket(socket, node, vmid, query, config))
}

async fn handle_socket(
    mut client_ws: WebSocket,
    node: String,
    vmid: String,
    query: VncQuery,
    config: Arc<Config>,
) {
    // 4. Construct Proxmox WebSocket URL
    let wss_url = format!(
        "{}/nodes/{}/qemu/{}/vncwebsocket?port={}&vncticket={}",
        config.proxmox_url.replace("http", "ws"),
        node,
        vmid,
        query.port,
        urlencoding::encode(&query.vncticket)
    );

    // 5. Connect to Proxmox WebSocket with Custom TLS Connector (Allow Invalid/Self-Signed Certs)
    let native_connector = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .unwrap();
    let connector = Connector::NativeTls(native_connector);

    let mut request =
        match tokio_tungstenite::tungstenite::client::IntoClientRequest::into_client_request(
            &wss_url,
        ) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("Invalid WS URL: {}", e);
                let _ = client_ws.close().await;
                return;
            }
        };

    let auth_val = format!(
        "PVEAPIToken={}={}",
        config.proxmox_token_id, config.proxmox_token_secret
    );
    request
        .headers_mut()
        .insert("Authorization", auth_val.parse().unwrap());

    let parsed_url = reqwest::Url::parse(&wss_url).unwrap();
    let host = parsed_url.host_str().unwrap();
    let port = parsed_url.port().unwrap_or(8006);

    let tcp_stream = match tokio::net::TcpStream::connect((host, port)).await {
        Ok(stream) => {
            let _ = stream.set_nodelay(true);
            stream
        }
        Err(e) => {
            eprintln!("Failed to create TCP stream: {}", e);
            let _ = client_ws.close().await;
            return;
        }
    };

    let (proxmox_ws, _) = match tokio_tungstenite::client_async_tls_with_config(
        request,
        tcp_stream,
        None,
        Some(connector),
    )
    .await
    {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("Failed to connect to Proxmox WS: {}", e);
            let _ = client_ws
                .send(axum::extract::ws::Message::Text(
                    format!("Failed to connect to Proxmox: {}", e).into(),
                ))
                .await;
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
                Message::Binary(b) => TungsteniteMessage::Binary(b),
                Message::Ping(p) => TungsteniteMessage::Ping(p),
                Message::Pong(p) => TungsteniteMessage::Pong(p),
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
                TungsteniteMessage::Binary(b) => Message::Binary(b),
                TungsteniteMessage::Ping(p) => Message::Ping(p),
                TungsteniteMessage::Pong(p) => Message::Pong(p),
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
