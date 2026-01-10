# --- Frontend Build Stage ---
FROM rust:1.81-slim-bookworm AS frontend-builder
WORKDIR /app
RUN apt-get update && apt-get install -i -y pkg-config libssl-dev curl Binaryen
RUN cargo install --locked trunk
RUN rustup target add wasm32-unknown-unknown
COPY . .
RUN trunk build --release

# --- Backend Build Stage ---
FROM rust:1.81-slim-bookworm AS backend-builder
WORKDIR /app
RUN apt-get update && apt-get install -i -y pkg-config libssl-dev
COPY . .
# Copy compiled frontend from previous stage
COPY --from=frontend-builder /app/dist /app/dist
RUN cargo build --release --bin uci-server

# --- Final Production Image ---
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -i -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary and assets
COPY --from=backend-builder /app/target/release/uci-server /app/uci-server
COPY --from=backend-builder /app/dist /app/dist

# Expose the application port
EXPOSE 3000

# Start command
CMD ["./uci-server"]
