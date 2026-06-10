# proxmox-vnc

High-performance WebSocket VNC Proxy for Proxmox Custom Dashboard, written in Rust (Tokio).

## Tech Stack
- **Language:** Rust
- **Runtime:** Tokio (async)
- **Role:** Bridges noVNC browser streams to Proxmox VE VNC endpoints securely

## Running Locally
```bash
cp .env.example .env
cargo run
```

## CI/CD
Automated pipeline via GitHub Actions:
1. 🧪 **Rustfmt & Clippy** (strict lint gate)
2. 🐳 **Docker Build & Push** → `ghcr.io/ginganomercy/proxmox-vnc`
3. 🔒 **Trivy Security Scan** (blocks CRITICAL CVEs)
4. 🚀 **Auto-Deploy** to Docker Swarm via Tailscale SSH
