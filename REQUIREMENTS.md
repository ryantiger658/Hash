# #ash — Requirements

> Pronounced "hash" — a self-hosted markdown knowledge base.
> The `#` is the markdown heading character (ASCII 35); the default port 3535 = `##`.

> Status: **Draft v0.4** — requirements complete; ready to scaffold

## Overview

An open-source, self-hosted markdown knowledge base with a server component (Docker) and lightweight desktop clients. Designed for homelab use with offline-first sync semantics.

---

## Goals

- Self-hosted, privacy-first knowledge base using plain markdown files
- No vendor lock-in: documents are stored as standard `.md` files on disk
- Works fully offline; syncs automatically when connectivity is restored
- Runs alongside existing homelab infrastructure (Portainer, SMB shares)

## Non-Goals (v1)

- Cloud hosting or managed SaaS
- Support for non-markdown document formats (PDF, DOCX, etc.)
- Real-time collaborative editing
- Multi-user accounts

---

## Architecture

```
[SMB Share / Document Repository]
        |
        | (volume mount)
        v
+-------------------+
|   Docker Server   |
|                   |
|  - REST / WebSocket API
|  - Web UI (read + edit markdown)
|  - Sync engine (server-side)
|  - Auth layer     |
+-------------------+
        |
        | (network / local)
        v
+-------------------+
|  Desktop Client   |
|                   |
|  - Sync agent     |
|  - (optional) Markdown viewer/editor
|  - Offline cache  |
+-------------------+
```

---

## Server Requirements

### Deployment

- Distributed as a single Docker image
- Managed via Portainer (standard Docker Compose compatible)
- Document storage via mounted SMB share (or any POSIX-compatible volume)
- Configurable via environment variables and/or a config file

### Web Interface

- Browse the document repository (folder tree + file list)
- Read markdown with rendered preview
- Edit markdown (in-browser editor with live preview)
- Create, rename, and delete files and folders
- Search across all documents (full-text)
- Responsive design (usable on mobile/tablet)

### Sync API

- Expose a sync protocol over HTTP/WebSocket for desktop clients
- Support delta sync (only transfer changed content)
- Track document versions to detect and handle conflicts
- Support multiple connected clients simultaneously

### Security

- API key authentication (configured in `config.toml` or via env var)
- HTTPS recommended via reverse proxy (e.g., Traefik, nginx); built-in TLS optional future addition
- Future: username/password login, OIDC/SSO support

---

## Desktop Client Requirements

### Core (Required)

- Sync markdown folders with the server
- Operate fully offline; queue changes locally
- Automatically sync when server connectivity is restored
- Conflict detection with user-facing resolution workflow
- Cross-platform: macOS, Windows, Linux
- Lightweight: minimal resource footprint, runs in system tray

### Editor (Stretch Goal)

- View rendered markdown offline
- Edit markdown with live preview
- Keyboard-friendly interface

---

## Sync Behavior

| Scenario | Expected Behavior |
|---|---|
| Client offline, edits locally | Changes queued; synced on reconnect |
| Server-side edit while client offline | Client pulls changes on reconnect |
| Conflicting edits (both sides changed) | Conflict flagged; user chooses resolution |
| New file on server | Pulled to client on next sync |
| File deleted on server | Marked for deletion on client (with confirmation option) |
| File deleted on client | Propagated to server on sync |

---

## Technology Decisions

| Concern | Decision | Rationale |
|---|---|---|
| Server language | Rust | Small Docker image, high performance, shared ecosystem with desktop |
| Desktop framework | Tauri (Rust + WebView) | Lightweight native app; avoids bundling a browser engine like Electron |
| Frontend (web + desktop) | Svelte | Compiles to plain JS, minimal runtime, shared between server web UI and Tauri shell |
| Desktop platforms | macOS, Windows, Linux | Tauri supports all three natively |
| Mobile (future) | Android | Tauri v2 supports Android; deferred to a later milestone |
| User model | Single-user | No multi-tenancy needed |
| Conflict resolution | Last-write-wins | Simple; no collaborative editing required |
| Sync style | Auto-save + auto-sync | Changes saved locally and pushed to server automatically |
| Sync protocol | REST-based delta (timestamp + checksum) | Simple to implement and debug; no CRDT complexity needed |
| Storage | Fully file-based (no database) | Transparent, inspectable, trivially backupable |
| Config format | TOML | Human-readable, simple, used for both server and client config |
| Sync metadata | `.mdkb/sync/<file>.toml` per tracked file | Stores checksum, modified time, last synced time alongside documents |
| Search index | In-memory, rebuilt on startup | No persistence needed; scanning a markdown vault is fast |
| Authentication | Single API key (in `config.toml`) | Simple for v1; auth options (username/password, OIDC) deferred |
| Default port | 3535 | ASCII 35 = `#`, the markdown heading character |
| SMB file watching | Polling every 30s (configurable) | inotify is unreliable over SMB |

---

## Markdown Support

- **Flavor:** CommonMark + GitHub Flavored Markdown (GFM)
  - Tables, task lists, strikethrough, fenced code blocks with syntax highlighting
- **Wiki-links:** `[[Note Name]]` for inter-note linking; resolves by filename, then by frontmatter `title`; ambiguous matches surface a picker
- **YAML frontmatter:** supported but optional; recommended fields: `title`, `tags`, `created`, `updated`
- **Attachments (future):** images and binary files linked from markdown; synced alongside their parent notes

## Document Organization

No enforced structure. Recommended conventions (documented, not required):

```
vault/
  assets/        # images and file attachments
  .mdkb/         # tool metadata — not user content, not synced to clients
    sync/        # per-file sync state TOML files
  **/*.md        # notes anywhere in the tree
```

## Resolved Decisions

| Question | Decision |
|---|---|
| API key setup UX | Env var + `config.toml`; generated with a placeholder on first run. No wizard needed for v1. |
| Desktop sync scope (v1) | Full vault sync only; folder selection deferred. |
| Attachment sync | Auto-include linked attachments when syncing a note. |

---

## Milestones (Proposed)

| Milestone | Scope |
|---|---|
| M0 — Foundations | Repo setup, Docker scaffold, SMB mount, basic file browsing API |
| M1 — Web UI | Read + edit markdown in browser, folder tree, search |
| M2 — Sync Protocol | Delta sync API, version tracking, conflict detection |
| M3 — Desktop Client (Sync) | Background sync agent, offline queue, auto-reconnect |
| M4 — Desktop Client (Editor) | Offline viewer + editor (stretch goal) |

---

## Revision History

| Version | Date | Notes |
|---|---|---|
| 0.1 | 2026-03-15 | Initial draft |
| 0.2 | 2026-03-15 | Tech stack decided; open questions narrowed |
| 0.3 | 2026-03-15 | Resolved storage, auth, port, markdown flavor, doc organization |
| 0.4 | 2026-03-15 | Closed all open questions; added AGENTS.md |
