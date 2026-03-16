# ── Stage 1: Build the Svelte UI ─────────────────────────────────────────────
FROM node:20-alpine AS ui-builder

WORKDIR /app/ui
COPY ui/package.json ui/package-lock.json* ./
RUN npm ci

COPY ui/ ./
# Build targeting the server's static directory
RUN BUILD_TARGET=server npm run build


# ── Stage 2: Build the Rust server ───────────────────────────────────────────
FROM rust:1.78-alpine AS server-builder

# musl-dev is required for static linking on Alpine
RUN apk add --no-cache musl-dev

WORKDIR /app/server
COPY server/Cargo.toml server/Cargo.lock* ./
COPY server/src ./src

# Copy the compiled UI so the server can embed the static/ dir check at build time
COPY --from=ui-builder /app/server/static ./static

RUN cargo build --release


# ── Stage 3: Minimal runtime image ───────────────────────────────────────────
FROM alpine:3.19

RUN apk add --no-cache ca-certificates

WORKDIR /app

# Copy the compiled binary
COPY --from=server-builder /app/server/target/release/hash-server ./hash-server

# Copy the compiled UI (served as static files at runtime)
COPY --from=ui-builder /app/server/static ./static

# Vault is mounted here at runtime
VOLUME ["/vault"]

# Default config location (override with HASH_CONFIG env var)
ENV HASH_CONFIG=/app/config.toml

EXPOSE 3535

CMD ["./hash-server"]
