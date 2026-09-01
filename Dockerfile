# Build Stage
FROM rust:alpine AS build

RUN apk add --no-cache musl-dev gcc make perl

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
FROM alpine:latest

RUN apk add --no-cache ca-certificates tzdata

WORKDIR /app

# Copy binary from build stage
COPY --from=build /usr/src/app/target/release/vnc-proxy .

EXPOSE 3002

CMD ["./vnc-proxy"]
