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

---

## Server Installation

### Docker Compose (recommended)

```yaml
services:
  hash:
    image: ghcr.io/ryantiger658/hash:latest
    ports:
      - "3535:3535"
    volumes:
      - hash-vault:/vault
      - ./config.toml:/app/config.toml:ro
    restart: unless-stopped

volumes:
  hash-vault:
```

Create a `config.toml` next to your compose file:

```toml
[auth]
api_key = "your-secret-key"   # change this

[ui]
secondary_color = "#aaff00"   # accent color (any CSS hex)
default_theme = "dark"        # "light", "dark", or "system"
```

Then start it:

```bash
docker compose up -d
```

Open `http://your-server:3535` in your browser and enter your API key to log in.

### Portainer

1. Go to **Stacks → Add stack**
2. Paste the Docker Compose above
3. Click **Deploy the stack**

---

## Desktop Client Installation

Download the latest release for your platform from the [Releases](../../releases) page:

| Platform | File |
|---|---|
| macOS (Apple Silicon + Intel) | `hash_0.0.x_universal.dmg` |
| Windows | `hash_0.0.x_x64-setup.exe` or `.msi` |
| Linux | `hash_0.0.x_amd64.AppImage` or `.deb` |

### macOS — Gatekeeper warning

Because the app is not yet notarized with Apple, macOS will show:

> *"hash.app" cannot be opened because Apple cannot verify it is free of malware.*

**To open it anyway:**

1. Right-click (or Control-click) `hash.app` → **Open**
2. Click **Open** in the dialog that appears

You only need to do this once. Subsequent launches open normally.

Alternatively, remove the quarantine flag from Terminal:

```bash
xattr -cr /Applications/hash.app
```

### First launch

1. Open the app — you will see the `#ash` login screen
2. Enter your **Server URL** (e.g. `http://192.168.1.100:3535`)
3. Enter your **API Key** from `config.toml`
4. Click **Connect** — your notes will appear

The server icon (⬜) in the sidebar footer shows sync status. Click it to configure the background sync:

| Color | Meaning |
|---|---|
| Grey | Not configured — click to set up |
| Green | Connected and synced |
| Amber | Connected, changes pending |
| Red | Server unreachable |

---

## Default Port

**3535** — because ASCII 35 is `#`, and `3535` = `##`, the H2 markdown heading.

## Theming

`#ash` supports light and dark mode with a configurable accent color. Set your preferred accent in `config.toml`:

```toml
[ui]
secondary_color = "#6366f1"   # any valid CSS hex color
default_theme = "dark"        # "light", "dark", or "system"
```

---

## Development

```
#ash/
  server/      # Rust server (Axum)
  ui/          # Svelte frontend (shared by server + desktop)
  desktop/     # Tauri v2 desktop client
  docker/      # Dockerfile + docker-compose.yml
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

### Run all tests

```bash
make ci
```

## License

[AGPL-3.0](LICENSE) — if you run a modified version as a network service, you must publish your changes.
