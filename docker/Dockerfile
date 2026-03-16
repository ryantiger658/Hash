# ── Stage 1: Build the Svelte UI ─────────────────────────────────────────────
# Chainguard Node image — minimal attack surface, near-zero CVEs
FROM cgr.dev/chainguard/node:latest-dev AS ui-builder

WORKDIR /app/ui
COPY ui/package.json ui/package-lock.json ./
RUN npm ci --ignore-scripts

COPY ui/ ./
# Build into server/static/ so the Rust stage can pick it up
RUN BUILD_TARGET=server npm run build


# ── Stage 2: Build the Rust server ───────────────────────────────────────────
# Chainguard Rust image (Wolfi/glibc-based)
FROM cgr.dev/chainguard/rust:latest-dev AS server-builder

USER root
WORKDIR /app

# Copy workspace manifests — Docker caches this layer until they change
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml ./server/

# Stub binary so cargo can pre-build all dependencies as a cache layer
RUN mkdir -p server/src && echo 'fn main() {}' > server/src/main.rs && echo '' > server/src/lib.rs
RUN cargo build --release -p hash-server
RUN rm -rf server/src

# Now build the real server
COPY server/src ./server/src
# Touch main.rs to force Cargo to re-link after the stub
RUN touch server/src/main.rs server/src/lib.rs && cargo build --release -p hash-server

# Copy compiled UI into the place the binary expects at runtime
COPY --from=ui-builder /app/server/static ./server/static

# Prepare vault seed — copied to /vault in the runtime image so that Docker
# initialises a new named volume with welcome content and nonroot ownership.
RUN mkdir -p /vault-seed
COPY dev/vault/Welcome.md dev/vault/Getting-Started.md /vault-seed/


# ── Stage 3: Minimal distroless runtime ──────────────────────────────────────
# Chainguard glibc-dynamic — provides glibc runtime, runs as nonroot by default
FROM cgr.dev/chainguard/glibc-dynamic:latest

WORKDIR /app

COPY --from=server-builder /app/target/release/hash-server ./hash-server
COPY --from=server-builder /app/server/static ./static

# Seed vault with welcome content, owned by nonroot (uid 65532).
# Docker copies this directory into a new named volume on first mount —
# welcome notes appear automatically and the nonroot user can write to the vault.
COPY --chown=65532:65532 --from=server-builder /vault-seed /vault

VOLUME ["/vault"]

ENV HASH_CONFIG=/app/config.toml

EXPOSE 3535

ENTRYPOINT ["/app/hash-server"]
