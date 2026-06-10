# 🔌 Proxmox VNC Proxy

A high-performance, strictly isolated Microservice dedicated to securing and tunneling VNC (noVNC) WebSocket streams from the Proxmox Custom Dashboard to your internal Proxmox VE servers.

## 🚀 Tech Stack

- **Rust (Edition 2024)**: Systems programming language guaranteeing memory safety and thread safety without a garbage collector. Essential for preventing memory leaks during heavy WebSocket video frame streaming.
- **Tokio**: The industry-standard asynchronous runtime for Rust. Provides extreme concurrency.
- **Axum**: A modern, ergonomic web framework built on top of Tokio.
- **tokio-tungstenite**: Lightweight async WebSocket library for handling the raw byte streams.
- **reqwest**: Fast HTTP client for acquiring VNC tickets from the Proxmox API.
- **jsonwebtoken**: For guarding the WebSocket endpoints. Only clients with a valid JWT from the `core-api` can initiate a stream.

---

## 📂 Folder Structure

```text
vnc-proxy/
├── .github/workflows/   # CI/CD Deployment pipelines (Trivy & Tailscale)
├── src/
│   ├── auth.rs          # JWT parsing and validation logic
│   ├── config.rs        # Environment variable ingestion (Dotenv)
│   ├── proxmox.rs       # Communicates with Proxmox API to request VNC tickets
│   ├── proxy.rs         # The core WebSocket streaming tunnel (Byte forwarder)
│   └── main.rs          # Axum Router setup and application entrypoint
├── Dockerfile           # Minimal multi-stage Rust build (distroless/scratch base)
├── Cargo.toml           # Rust package manifest and dependencies
└── Cargo.lock           # Dependency lockfile
```

---

## 🛠️ Local Development Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/ginganomercy/proxmox-vnc.git
   cd proxmox-vnc
   ```

2. **Prepare Environment Variables:**
   ```bash
   cp .env.example .env
   # Ensure the PROXMOX_URL and JWT_SECRET match your core-api
   ```

3. **Run with Cargo:**
   ```bash
   cargo run
   ```
   *The WebSocket proxy will listen on `ws://localhost:3002/api/vnc`.*

---

## 🔒 CI/CD & Deployment

This service utilizes an **Enterprise-Grade GitHub Actions Pipeline**:
1. **Strict Linting**: Gates code quality using `cargo fmt` and `cargo clippy`. Unsafe or unoptimized code will fail the build.
2. **Docker Build**: Compiles the Rust binary and packages it into a tiny container image (`ghcr.io`).
3. **DevSecOps**: Scans the Docker image using **Trivy** to ensure zero known CVE vulnerabilities.
4. **Zero-Trust Deployment**: Automatically deploys the updated container to Docker Swarm via a private **Tailscale** tunnel, ensuring your server remains closed off from the public internet.
