# #ash — Makefile
# Common targets for local development, building, and testing.
#
# Quick start:
#   make setup   ← one-time first-run setup
#   make dev     ← start server + UI dev server together

# ── Config ────────────────────────────────────────────────────────────────────
DEV_CONFIG  := dev/config.toml
DEV_VAULT   := dev/vault
DOCKER_TAG  := hashash:latest
DOCKER_FILE := docker/Dockerfile

# Pass the dev config to the server automatically.
export HASH_CONFIG := $(DEV_CONFIG)

# Detect OS for platform-specific behaviour (e.g. opening a browser).
UNAME := $(shell uname)

# ── Help (default target) ─────────────────────────────────────────────────────
.DEFAULT_GOAL := help
.PHONY: help
help:
	@echo ""
	@echo "  #ash — available targets"
	@echo ""
	@echo "  Setup"
	@echo "    setup            First-time setup: install deps + create dev config"
	@echo "    install          Install all npm dependencies (ui + desktop)"
	@echo ""
	@echo "  Development"
	@echo "    dev              Run server + UI dev server together (Ctrl-C to stop both)"
	@echo "    server           Run the server only (uses dev/config.toml)"
	@echo "    ui               Run the Svelte dev server only (port 5173)"
	@echo "    desktop          Run the Tauri desktop app in dev mode"
	@echo ""
	@echo "  Build"
	@echo "    build            Build server binary + UI static files"
	@echo "    build-server     Build the Rust server binary (release)"
	@echo "    build-ui         Build the Svelte UI into server/static/"
	@echo "    build-desktop    Build the Tauri desktop app (release)"
	@echo "    build-docker     Build the Docker image ($(DOCKER_TAG))"
	@echo ""
	@echo "  Test & Lint"
	@echo "    test             Run all Rust tests"
	@echo "    lint             Run clippy + rustfmt check"
	@echo "    fmt              Auto-format all Rust code"
	@echo "    check            Fast compile check (no binary output)"
	@echo ""
	@echo "  Docker"
	@echo "    docker-up        Start #ash via docker-compose"
	@echo "    docker-down      Stop docker-compose"
	@echo "    docker-logs      Tail docker-compose logs"
	@echo ""
	@echo "  Housekeeping"
	@echo "    clean            Remove all build artifacts"
	@echo "    clean-server     Remove server build artifacts"
	@echo "    clean-ui         Remove UI dist and node_modules"
	@echo ""


# ── Setup ─────────────────────────────────────────────────────────────────────
.PHONY: setup
setup: install dev-config
	@echo ""
	@echo "  Setup complete!"
	@echo "  Edit $(DEV_CONFIG) — set vault.path and auth.api_key, then run: make dev"
	@echo ""

.PHONY: dev-config
dev-config:
	@if [ ! -f $(DEV_CONFIG) ]; then \
		cp server/config.example.toml $(DEV_CONFIG); \
		echo "  Created $(DEV_CONFIG) from example — edit it before running."; \
		sed -i.bak 's|path = "/vault"|path = "$(PWD)/$(DEV_VAULT)"|g' $(DEV_CONFIG) && rm -f $(DEV_CONFIG).bak; \
		echo "  Vault path auto-set to $(PWD)/$(DEV_VAULT)"; \
	else \
		echo "  $(DEV_CONFIG) already exists — skipping."; \
	fi

.PHONY: install
install: ui-install desktop-install

.PHONY: ui-install
ui-install:
	cd ui && npm install

.PHONY: desktop-install
desktop-install:
	cd desktop && npm install


# ── Development ───────────────────────────────────────────────────────────────

# Run server and UI dev server together.
# Both processes are killed when you press Ctrl-C.
.PHONY: dev
dev: dev-config
	@echo "  Starting #ash server + UI dev server..."
	@echo "  Server  → http://localhost:3535"
	@echo "  UI dev  → http://localhost:5173  (proxies /api to server)"
	@echo "  Press Ctrl-C to stop both."
	@echo ""
	@trap 'kill 0' SIGINT SIGTERM; \
		~/.cargo/bin/cargo run -p hash-server & \
		(cd ui && npm run dev) & \
		wait

.PHONY: server
server: dev-config
	@echo "  Starting #ash server on http://localhost:3535"
	~/.cargo/bin/cargo run -p hash-server

.PHONY: ui
ui:
	cd ui && npm run dev

.PHONY: desktop
desktop:
	@which cargo-tauri > /dev/null 2>&1 || ~/.cargo/bin/cargo install tauri-cli --version "^2"
	cd desktop && npm run tauri dev


# ── Build ─────────────────────────────────────────────────────────────────────
.PHONY: build
build: build-ui build-server

.PHONY: build-server
build-server:
	~/.cargo/bin/cargo build --release -p hash-server

.PHONY: build-ui
build-ui:
	cd ui && BUILD_TARGET=server npm run build

.PHONY: build-desktop
build-desktop:
	@which cargo-tauri > /dev/null 2>&1 || ~/.cargo/bin/cargo install tauri-cli --version "^2"
	cd desktop && npm run build

.PHONY: build-docker
build-docker:
	docker build -f $(DOCKER_FILE) -t $(DOCKER_TAG) .


# ── Test & Lint ───────────────────────────────────────────────────────────────
.PHONY: test
test:
	~/.cargo/bin/cargo test

.PHONY: lint
lint:
	~/.cargo/bin/cargo fmt --check
	~/.cargo/bin/cargo clippy -- -D warnings

.PHONY: fmt
fmt:
	~/.cargo/bin/cargo fmt

.PHONY: check
check:
	~/.cargo/bin/cargo check


# ── Docker ────────────────────────────────────────────────────────────────────
.PHONY: docker-up
docker-up:
	docker compose -f docker/docker-compose.yml up -d
	@echo "  #ash running at http://localhost:3535"

.PHONY: docker-down
docker-down:
	docker compose -f docker/docker-compose.yml down

.PHONY: docker-logs
docker-logs:
	docker compose -f docker/docker-compose.yml logs -f


# ── Housekeeping ──────────────────────────────────────────────────────────────
.PHONY: clean
clean: clean-server clean-ui

.PHONY: clean-server
clean-server:
	~/.cargo/bin/cargo clean

.PHONY: clean-ui
clean-ui:
	rm -rf ui/dist ui/node_modules server/static
