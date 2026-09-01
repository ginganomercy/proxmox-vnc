# Build Stage
FROM rust:1.80-slim-bookworm AS build

RUN apt-get update && \
    apt-get install -y --no-install-recommends build-essential pkg-config perl && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# === STEP 1: LAYER CACHING TRICK ===
# Only copy Cargo manifests first
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to fool Cargo into compiling the dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# === STEP 2: BUILD ACTUAL SOURCE ===
# Now copy the actual source code. If only src/ changes, Docker uses the cached dependencies!
COPY src ./src

# We MUST update the timestamp of the actual main.rs to force cargo to rebuild our binary,
# otherwise it thinks the dummy build is up-to-date.
RUN touch src/main.rs

# Build the release binary
RUN cargo build --release

# Production Stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates tzdata && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from build stage
COPY --from=build /usr/src/app/target/release/vnc-proxy .

EXPOSE 3002

CMD ["./vnc-proxy"]
