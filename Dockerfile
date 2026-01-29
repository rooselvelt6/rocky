# --- Frontend Build Stage ---
FROM rust:1.93-slim-bookworm AS frontend-builder
WORKDIR /app

# Install dependencies and Trunk
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    pkg-config libssl-dev curl binaryen build-essential && \
    curl -L https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- -C /usr/local/bin && \
    rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-unknown-unknown
COPY . .
RUN trunk build --release

# --- Backend Build Stage ---
FROM rust:1.93-slim-bookworm AS backend-builder
WORKDIR /app

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    pkg-config libssl-dev musl-tools build-essential && \
    rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

COPY . .
COPY --from=frontend-builder /app/dist /app/dist

# Build statically linked binary
RUN cargo build --release --target x86_64-unknown-linux-musl --bin uci-server

# --- Final Production Image ---
FROM alpine:latest
WORKDIR /app

# Install runtime dependencies
RUN apk add --no-cache ca-certificates tzdata wget

# Create non-root user for security
RUN addgroup -g 1000 rocky && \
    adduser -D -u 1000 -G rocky rocky && \
    chown -R rocky:rocky /app

# Copy the static binary and assets
COPY --from=backend-builder --chown=rocky:rocky /app/target/x86_64-unknown-linux-musl/release/uci-server /app/uci-server
COPY --from=backend-builder --chown=rocky:rocky /app/dist /app/dist

# Switch to non-root user
USER rocky

# Expose the application port
EXPOSE 3000

# Metadata
ENV DB_HOST=surrealdb \
    DB_PORT=8000 \
    RUST_LOG=info

# Internal health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=40s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/api/health || exit 1

# Start command
CMD ["./uci-server"]
