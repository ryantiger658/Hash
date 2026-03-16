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

WORKDIR /app/server

# Copy manifests first — Docker caches this layer until Cargo.toml/lock changes
COPY server/Cargo.toml server/Cargo.lock ./

# Stub binary so cargo can pre-build all dependencies as a cache layer
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Now build the real server
COPY server/src ./src
# Touch main.rs to force Cargo to re-link after the stub
RUN touch src/main.rs && cargo build --release

# Copy compiled UI into the place the binary expects at runtime
COPY --from=ui-builder /app/server/static ./static


# ── Stage 3: Minimal distroless runtime ──────────────────────────────────────
# Chainguard glibc-dynamic — provides glibc runtime, runs as nonroot by default
FROM cgr.dev/chainguard/glibc-dynamic:latest

WORKDIR /app

COPY --from=server-builder /app/server/target/release/hash-server ./hash-server
COPY --from=server-builder /app/server/static ./static

# Vault is mounted here at runtime (e.g. SMB share via Portainer)
VOLUME ["/vault"]

ENV HASH_CONFIG=/app/config.toml

EXPOSE 3535

ENTRYPOINT ["/app/hash-server"]
