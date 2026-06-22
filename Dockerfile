# Build Stage
FROM rust:alpine AS build

RUN apk add --no-cache musl-dev gcc pkgconf openssl-dev

WORKDIR /usr/src/app

# Copy files
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the release binary
RUN cargo build --release

# Production Stage
FROM alpine:latest

RUN apk add --no-cache ca-certificates tzdata openssl

WORKDIR /app

# Copy binary from build stage
COPY --from=build /usr/src/app/target/release/vnc-proxy .

EXPOSE 3002

CMD ["./vnc-proxy"]
