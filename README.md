# #ash

> Pronounced "hash" — a self-hosted, offline-first markdown knowledge base.

`#ash` lets you create, edit, and sync markdown documents across devices. It runs as a Docker container in your homelab and syncs with lightweight desktop clients that work fully offline.

Notes support Obsidian-style frontmatter tags, wiki-links, Mermaid diagrams, and a distraction-free focus mode.

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

## MCP and OKF

#ash exposes an authenticated, stateless Streamable HTTP MCP server at `https://your-server/mcp`. Configure your MCP client with that URL and the same API key as a Bearer token. It provides `search_notes` and `read_note`; search results include a URL that opens the original note in #ash.

When deployed behind a reverse proxy, set the public URL so those source links are stable:

```toml
[server]
public_url = "https://notes.example.com"
```

OKF v0.1 support is recognition-only: any ordinary Markdown note with leading YAML frontmatter and a non-empty `type` field is recognized as an OKF concept. `index.md` and `log.md` are treated as OKF reserved files. Existing notes are never changed or rejected, and no `validate_okf` MCP tool is exposed.

## OpenID Connect login

The web interface supports any standards-compliant OpenID Connect provider. Register this callback URL with the provider:

```text
https://notes.example.com/api/auth/oidc/callback
```

When running without a `config.toml`, configure the server with environment variables:

```yaml
environment:
  HASH_API_KEY: "your-automation-api-key"
  HASH_PUBLIC_URL: "https://notes.example.com"
  HASH_OIDC_ISSUER: "https://identity.example.com/oidc"
  HASH_OIDC_CLIENT_ID: "hash"
  HASH_OIDC_CLIENT_SECRET: "replace-with-provider-secret"
  HASH_OIDC_REDIRECT_URL: "https://notes.example.com/api/auth/oidc/callback"
  HASH_OIDC_SCOPES: "openid profile email"
```

The issuer must expose OpenID Connect discovery metadata. The browser uses Authorization Code with PKCE and receives an `HttpOnly` session cookie after the ID token is verified. Browser sessions last 24 hours and are cleared when the server restarts. HTTPS deployments use the host-only `__Host-hash-session-v2` cookie; #ash automatically removes the legacy `hash-session` cookie during migration. The API key remains available for desktop sync, MCP clients, and administrator fallback access.

If you mount a `config.toml`, use the equivalent `[auth]` fields shown in [`config.example.toml`](config.example.toml); the project loads the config file instead of environment configuration.

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

Because the app is not notarized with Apple, macOS Gatekeeper will block it on first launch:

> *"hash.app" cannot be opened because Apple cannot verify it is free of malware.*

**macOS Sequoia (15+):**

1. Click **Done** to dismiss the dialog
2. Open **System Settings → Privacy & Security**
3. Scroll down to the Security section — you will see *"hash.app was blocked"*
4. Click **Open Anyway** → enter your password → click **Open Anyway** again

**macOS Ventura / Sonoma (13–14):**

1. Right-click (or Control-click) `hash.app` → **Open**
2. Click **Open** in the confirmation dialog

**Alternative (any macOS version) — Terminal:**

```bash
xattr -cr /Applications/hash.app
```

You only need to do this once. Subsequent launches open normally.

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
