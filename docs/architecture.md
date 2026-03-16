# #ash Architecture

## Overview

#ash has three components that work together:

```
[SMB Share / Vault]
       |
       | (Docker volume mount)
       v
  ┌─────────────┐          ┌─────────────────┐
  │   Server    │◄─────────│  Desktop Client │
  │  (Rust)     │  REST    │  (Tauri + Svelte)│
  │  port 3535  │          │  macOS/Win/Linux │
  └──────┬──────┘          └─────────────────┘
         │
         │ serves
         v
  ┌─────────────┐
  │   Web UI    │
  │  (Svelte)   │
  │  browser    │
  └─────────────┘
```

## Server (`server/`)

A Rust HTTP server built on [Axum](https://github.com/tokio-rs/axum).

**Responsibilities:**
- Serve the compiled Svelte UI as static files
- Expose a REST API for file CRUD and sync
- Poll the vault directory for external changes (e.g. from direct SMB edits)
- Authenticate requests via API key

**Key modules:**
| File | Purpose |
|---|---|
| `src/main.rs` | Startup, config loading, server bind |
| `src/config.rs` | TOML config deserialization |
| `src/vault.rs` | Vault file operations + path traversal protection |
| `src/sync.rs` | Sync data types (snapshot, push payload) |
| `src/routes/auth.rs` | API key middleware |
| `src/routes/files.rs` | File CRUD handlers |
| `src/routes/sync.rs` | Sync API handlers |

## Frontend (`ui/`)

A [Svelte](https://svelte.dev) + [Vite](https://vitejs.dev) app. The same build is used in two contexts:

- **Browser:** served as static files by the Rust server (`BUILD_TARGET=server npm run build` → `server/static/`)
- **Desktop:** embedded in the Tauri window (`npm run build` → `ui/dist/`, referenced by `tauri.conf.json`)

During development, `vite dev` proxies `/api` requests to `localhost:3535`.

## Desktop Client (`desktop/`)

A [Tauri v2](https://tauri.app) application. The Svelte UI runs in a native webview; the Rust backend (`src-tauri/`) handles:

- Config storage (`~/.config/hash/config.toml`)
- Background sync against the server's `/api/sync/` endpoints
- Offline change queue (local writes while disconnected)
- Auto-reconnect and sync on network restoration

**Tauri commands (Rust → JS bridge):**
| Command | Description |
|---|---|
| `get_sync_status` | Returns connection state, last sync time, pending changes |
| `trigger_sync` | Runs an immediate full sync |

## Sync Protocol

### Delta sync flow

1. Desktop calls `GET /api/sync/snapshot` → receives list of all vault files with checksums + timestamps
2. Desktop compares snapshot to local file state
3. Desktop calls `POST /api/sync/push` with:
   - `upsert`: files where local checksum differs from server (base64-encoded content)
   - `delete`: files present locally that are absent from snapshot
4. Desktop pulls any files in the snapshot that are absent or outdated locally via `GET /api/files/*path`

### Conflict resolution

Last-write-wins. The client's version always wins on push. If a file was edited on both the server (e.g. via web UI) and the client while offline, the client's version is authoritative when it next syncs.

Future versions may add a conflict picker UI.

### Offline behavior

Changes made while the server is unreachable are held in a local pending queue (file-based, in the local vault). On reconnect, the sync flow runs automatically.

## File Layout

```
vault/
  .mdkb/           # #ash metadata — not shown in UI, not synced to clients
    sync/          # Per-file sync state: <sha256>.toml (checksum, modified, synced)
  assets/          # Recommended location for images and attachments
  **/*.md          # User notes — any directory structure
```

## Configuration

### Server (`config.toml`)

See `server/config.example.toml` for a fully commented example.

### Desktop (`~/.config/hash/config.toml`)

| Field | Description |
|---|---|
| `server_url` | URL of the #ash server |
| `api_key` | Must match server's `[auth] api_key` |
| `local_vault_path` | Local directory to sync the vault into |
| `sync_interval_secs` | Auto-sync frequency (default: 60) |
