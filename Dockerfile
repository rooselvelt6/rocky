# --- Shared Base Stage ---
FROM rust:1.93-slim-bookworm AS base
WORKDIR /app
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
    pkg-config libssl-dev curl binaryen build-essential musl-tools && \
    curl -L https://github.com/trunk-rs/trunk/releases/latest/download/trunk-x86_64-unknown-linux-gnu.tar.gz | tar -xzf- -C /usr/local/bin && \
    rm -rf /var/lib/apt/lists/*

# --- Frontend Build Stage ---
FROM base AS frontend-builder
RUN rustup target add wasm32-unknown-unknown
COPY . .
# Disable bulk-memory to avoid validation errors with older/strict WASM validators
# and skip wasm-opt for speed/compatibility
ENV RUSTFLAGS="-C target-feature=-bulk-memory"
RUN trunk build --release --no-wasm-opt && \
    mkdir -p dist && \
    sed -i 's/integrity="[^"]*"//g' dist/index.html 2>/dev/null || true && \
    sed -i 's/crossorigin="anonymous"//g' dist/index.html 2>/dev/null || true && \
    cp style.css dist/ 2>/dev/null || true

# --- Backend Build Stage ---
FROM base AS backend-builder
RUN rustup target add x86_64-unknown-linux-musl
COPY . .
COPY --from=frontend-builder /app/dist /app/dist
# Build statically linked binary
RUN cargo build --release --target x86_64-unknown-linux-musl --bin uci-server

# --- Final Production Image ---
FROM alpine:latest
WORKDIR /app
RUN apk add --no-cache ca-certificates tzdata wget
RUN addgroup -g 1000 rocky && \
    adduser -D -u 1000 -G rocky rocky && \
    chown -R rocky:rocky /app

COPY --from=backend-builder --chown=rocky:rocky /app/target/x86_64-unknown-linux-musl/release/uci-server /app/uci-server
COPY --from=backend-builder --chown=rocky:rocky /app/dist /app/dist

USER rocky
EXPOSE 3000
ENV DB_HOST=surrealdb DB_PORT=8000 RUST_LOG=info
HEALTHCHECK --interval=30s --timeout=3s --start-period=40s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:3000/api/health || exit 1
CMD ["./uci-server"]
