# --- Frontend Build Stage ---
FROM rust:1.81-slim-bookworm AS frontend-builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev curl Binaryen
RUN cargo install --locked trunk
RUN rustup target add wasm32-unknown-unknown
COPY . .
RUN trunk build --release

# --- Backend Build Stage ---
FROM rust:1.81-slim-bookworm AS backend-builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev musl-tools
RUN rustup target add x86_64-unknown-linux-musl

COPY . .
COPY --from=frontend-builder /app/dist /app/dist

# Build statically linked binary
RUN cargo build --release --target x86_64-unknown-linux-musl --bin uci-server

# --- Final Production Image ---
FROM alpine:latest
WORKDIR /app
RUN apk add --no-cache ca-certificates tzdata

# Copy the static binary and assets
COPY --from=backend-builder /app/target/x86_64-unknown-linux-musl/release/uci-server /app/uci-server
COPY --from=backend-builder /app/dist /app/dist

# Expose the application port
EXPOSE 3000

# Metadata
ENV DB_HOST=surrealdb:8000
ENV RUST_LOG=info

# Start command
CMD ["./uci-server"]
