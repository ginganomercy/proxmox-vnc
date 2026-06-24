# CBT VNC Proxy — WebSocket VNC Bridge

**Rust + Axum high-performance WebSocket relay for Proxmox VM console access**

---

## Overview

The VNC Proxy bridges the browser's WebSocket connection to the Proxmox noVNC WebSocket endpoint, enabling native in-browser VM console access. It:

1. Validates the incoming client JWT token
2. Fetches a Proxmox VNC ticket from the Core API
3. Opens a WebSocket connection to the Proxmox node's VNC port
4. Relays binary frames bidirectionally between client and Proxmox

---

## Tech Stack

| Component | Technology |
| :--- | :--- |
| Language | Rust (2024 edition) |
| HTTP/WS Framework | Axum 0.8.9 |
| Async Runtime | Tokio 1.52.3 (full features) |
| JWT Validation | `jsonwebtoken` 10.4.0 |
| WebSocket Client | `tokio-tungstenite` 0.29.0 (native-tls) |
| HTTP Client | `reqwest` 0.13.4 |
| Logging | `tracing` + `tracing-subscriber` |

---

## How It Works

```
Browser (noVNC JS)
    │  WebSocket Upgrade (wss://vnc.pbjt.web.id/?token=<JWT>&vmid=<id>)
    ▼
CBT VNC Proxy (Rust/Axum)
    │  1. Validate JWT (HS256 with shared JWT_SECRET)
    │  2. POST to Core API → /proxmox/nodes/:node/:type/:vmid/vncproxy
    │  3. Connect to Proxmox wss://<node>:5900/?...
    │  4. Relay frames bidirectionally
    ▼
Proxmox QEMU VNC Server
```

---

## Environment Variables

```env
JWT_SECRET=<must-match-core-api-secret>
PROXMOX_HOST=https://proxmox.pbjt.web.id:8006
PROXMOX_TOKEN_ID=root@pam!mytoken
PROXMOX_TOKEN_SECRET=<token-secret>
ALLOWED_ORIGIN=https://cloud-dashboard.pbjt.web.id
```

---

## Local Development

```bash
cp .env.example .env
cargo run
# WebSocket server listening on :3002
```

## Production Build

```bash
cargo build --release
# Binary: target/release/vnc-proxy
```

## Docker

```bash
docker build -t ghcr.io/ginganomercy/vnc-proxy:latest .
docker run --env-file .env -p 3002:3002 ghcr.io/ginganomercy/vnc-proxy:latest
```

---

## Security

| Control | Implementation |
| :--- | :--- |
| **JWT Validation** | Every WebSocket upgrade validates the Bearer token against the shared `JWT_SECRET` |
| **CORS** | `ALLOWED_ORIGIN` env restricts WebSocket origin to production frontend only |
| **TLS** | Connections to Proxmox use `native-tls` (vendored) |
| **No credential storage** | Proxmox VNC tickets are ephemeral — fetched per-session, not stored |

---

## CI/CD

On every push to `main`:

1. **Cargo build check**
2. **Build & Push** → `ghcr.io/ginganomercy/vnc-proxy:latest`
3. **Trivy Security Scan** — blocks on CRITICAL CVEs
4. **Deploy to Swarm** — via Tailscale + SSH → `docker service update --force`
