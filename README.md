# #ash

> Pronounced "hash" — a self-hosted, offline-first markdown knowledge base.

`#ash` lets you create, edit, and sync markdown documents across devices. It runs as a Docker container in your homelab and syncs with lightweight desktop clients that work fully offline.

The `#` in the name is the markdown heading character — the foundational symbol of any structured document.

## Architecture

| Component | Technology | Purpose |
|---|---|---|
| Server | Rust + Axum | REST API, web UI, sync engine |
| Frontend | Svelte | Shared UI for both web and desktop |
| Desktop | Tauri v2 | Lightweight native client with offline sync |
| Storage | Flat files | Markdown lives as plain `.md` files — no database |
| Config | TOML | Human-readable configuration |

## Quick Start (Docker)

```bash
# Copy and edit the example config
cp server/config.example.toml config.toml

# Start with Docker Compose
docker compose -f docker/docker-compose.yml up -d
```

Then open `http://localhost:3535` in your browser.

## Desktop Client

Download the latest release for your platform (macOS, Windows, Linux) from the [Releases](../../releases) page.

On first launch, enter your server URL and API key from `config.toml`.

## Default Port

**3535** — because ASCII 35 is `#`, and `3535` = `##`, the H2 markdown heading.

## Theming

`#ash` supports light and dark mode with a configurable accent color. Set your preferred accent in `config.toml`:

```toml
[ui]
secondary_color = "#6366f1"   # any valid CSS hex color
default_theme = "dark"        # "light", "dark", or "system"
```

## Development

See [docs/architecture.md](docs/architecture.md) for a full overview.

```
#ash/
  server/      # Rust server (Axum)
  ui/          # Svelte frontend (shared by server + desktop)
  desktop/     # Tauri v2 desktop client
  docker/      # Dockerfile + docker-compose.yml
  docs/        # Architecture, API reference
```

### Prerequisites

- Rust (stable) + Cargo — install via [rustup.rs](https://rustup.rs)
- Node.js 20+
- Tauri v2 CLI: `cargo install tauri-cli --version "^2"`

### Run the server

```bash
cd server
cargo run
```

### Run the UI (dev)

```bash
cd ui
npm install
npm run dev
```

### Run the desktop client (dev)

```bash
cd desktop
npm install
npm run tauri dev
```

## License

[AGPL-3.0](LICENSE) — if you run a modified version as a network service, you must publish your changes.
