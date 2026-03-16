# #ash — Requirements

> Pronounced "hash" — a self-hosted markdown knowledge base.
> The `#` is the markdown heading character (ASCII 35); the default port 3535 = `##`.

> Status: **v0.6** — M0 and M1 complete; v0.0.3 release candidate; M2 (Sync Protocol) is next

---

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
|  - REST API       |
|  - Web UI         |
|  - Sync engine    |
|  - Auth layer     |
+-------------------+
        |
        | (HTTP over network)
        v
+-------------------+
|  Desktop Client   |
|                   |
|  - Sync agent     |
|  - Markdown editor|
|  - Offline cache  |
+-------------------+
```

---

## Server Requirements

### Deployment

- Distributed as a single Docker image (`ghcr.io/ryantiger658/hash`)
- Managed via Portainer (standard Docker Compose compatible)
- Document storage via mounted volume (SMB share or any POSIX-compatible path)
- Configurable via environment variables **or** a `config.toml` file (env vars take precedence when no config file is present)
- Docker named volume seeds welcome content on first run
- Chainguard base images (near-zero CVE)

### Web Interface ✅ M1 + v0.0.3

- Browse the document repository (folder tree + file list); empty directories visible
- Read markdown with rendered preview
- Edit markdown (split editor: textarea + live preview)
- Edit / Split / Preview mode toggle — floating panel on the right edge; text labels opt-in via `editor_labels` config
- Optional line numbers in the editor pane (`line_numbers` config)
- Create and delete files and folders; delete button appears on hover only
- Full-text search across all `.md` files — filename matches ranked first, then content matches (case-insensitive, returns path + snippet + line number)
- Auto-save (1.5s debounce after last keystroke); `⌘S` manual save
- Save status dot: always-visible indicator; pulsates amber when unsaved, chartreuse when synced, red on error
- New note modal supporting nested paths (`folder/note.md`)
- Light / dark / system theme; accent color configurable server-side; accent auto-darkened in light mode for legibility
- Default accent color: chartreuse (`#aaff00`)
- Wiki-links: `[[Note Name]]` and `[[Note|Label]]`; alias resolution via frontmatter `aliases` field
- YAML frontmatter: Obsidian-compatible subset parser; `tags`, `aliases`, and arbitrary properties displayed above document content
- Daily journal: calendar button in sidebar toolbar; auto-opens `journal/YYYY/Mon/MM-DD-YYYY.md`; created if missing; auto-deleted on navigate-away if still empty
- File metadata footer (created date, last updated) rendered below each note
- Hidden files and directories (dotfiles) hidden by default; controllable via `show_hidden_files` config
- Login gate (API key); key stored in browser `localStorage`

### Sync API ✅ M1 (foundation)

- `GET /api/sync/snapshot` — returns full file list with checksums and timestamps
- `POST /api/sync/push` — accepts upserts (base64 content) and deletes; reports rejections
- Delta sync fields (`modified`, `checksum`) in place; full protocol deferred to M2

### Security

- Bearer token auth on all protected routes; public route for UI config (`/api/ui-config`)
- API key configured via `config.toml` or `HASH_API_KEY` env var
- Path traversal protection (vault root canonicalization + normalize)
- HTTPS recommended via reverse proxy (Traefik, nginx); built-in TLS deferred
- CVE scanning with Grype in CI; Chainguard images in production

---

## Desktop Client Requirements

### Core (M3)

- Sync markdown folders with the server
- Operate fully offline; queue changes locally
- Automatically sync when server connectivity is restored
- Conflict detection with user-facing resolution workflow
- Cross-platform: macOS, Windows, Linux
- Lightweight: minimal resource footprint, runs in system tray

### Editor (M4 — Stretch Goal)

- View rendered markdown offline
- Edit markdown with live preview
- Keyboard-friendly interface

---

## Configuration

| Field | Env var | Default | Notes |
|---|---|---|---|
| `server.host` | `HASH_HOST` | `0.0.0.0` | |
| `server.port` | `HASH_PORT` | `3535` | |
| `vault.path` | `HASH_VAULT_PATH` | *(required)* | |
| `vault.poll_interval_secs` | `HASH_POLL_INTERVAL` | `30` | |
| `auth.api_key` | `HASH_API_KEY` | *(required)* | |
| `ui.secondary_color` | `HASH_SECONDARY_COLOR` | `#aaff00` | Any CSS hex color |
| `ui.default_theme` | `HASH_DEFAULT_THEME` | `system` | `light`, `dark`, `system` |
| `ui.editor_labels` | `HASH_EDITOR_LABELS` | `false` | `true` to show Edit/Split/Preview text labels |
| `ui.show_hidden_files` | `HASH_SHOW_HIDDEN_FILES` | `false` | `true` to show dotfiles in the file tree |
| `ui.line_numbers` | `HASH_LINE_NUMBERS` | `false` | `true` to show line numbers in the editor |
| `ui.spell_check` | `HASH_SPELL_CHECK` | `false` | `true` to enable browser spell-check in the editor |

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
| Server language | Rust | Small Docker image, high performance |
| Desktop framework | Tauri v2 (Rust + WebView) | Lightweight; avoids bundling a browser engine |
| Frontend (web + desktop) | Svelte | Minimal runtime; shared between server web UI and Tauri shell |
| Desktop platforms | macOS, Windows, Linux | Tauri supports all three natively |
| Mobile (future) | Android | Tauri v2 supports Android; deferred |
| User model | Single-user | No multi-tenancy needed |
| Conflict resolution | Last-write-wins | Simple; no collaborative editing |
| Sync protocol | REST delta (timestamp + checksum) | Simple to implement; no CRDT complexity |
| Storage | File-based (no database) | Transparent, inspectable, trivially backupable |
| Config format | TOML + env var fallback | Human-readable; env vars enable Docker-native deployment |
| Sync metadata | `.mdkb/sync/<file>.toml` | Per-file tracking alongside documents |
| Search index | In-memory, rebuilt on startup | No persistence needed; markdown vaults are small |
| Authentication | Single API key | Simple for v1 homelab use |
| Default port | 3535 | ASCII 35 = `#`; 3535 = `##` |
| SMB file watching | 30s polling | inotify unreliable over SMB |
| Docker base image | Chainguard glibc-dynamic | Near-zero CVE distroless runtime |
| Container registry | GitHub Container Registry (ghcr.io) | Free for public repos; built into CI |

---

## Note Format Specification

Notes are plain UTF-8 text files. This format is a stable contract — nothing in this section changes without a migration.

| Element | Specification |
|---|---|
| Encoding | UTF-8, Unix line endings preferred |
| Extension | `.md` (required for search and rendering) |
| Frontmatter | Optional YAML block between `---` markers at the very start of the file |
| Body | CommonMark + GFM (tables, task lists, strikethrough, fenced code blocks) |
| Wiki-links | `[[Note Name]]` and `[[Note Name\|Label]]` |
| Attachments | Any file in the vault; referenced by relative path in standard markdown image/link syntax |

### Frontmatter fields (Obsidian-compatible subset)

| Field | Type | Notes |
|---|---|---|
| `title` | string | Display name; used for wiki-link resolution |
| `tags` / `tag` | string or array | Rendered as chips in the viewer |
| `aliases` / `alias` | string or array | Alternate names for wiki-link resolution |
| `created` | date string | Informational; not parsed by the server |
| `updated` | date string | Informational; not parsed by the server |
| *(any other key)* | scalar or array | Displayed as key-value pairs in the properties panel |

---

## Backwards Compatibility Policy

#ash treats note files as the source of truth. The format must remain readable by older and newer versions alike.

### Rules

1. **Notes are always backwards-compatible.** A note written for v0.0.1 must render correctly in all future versions without modification.
2. **New frontmatter fields are always optional.** Older clients that do not recognise a field must ignore it gracefully.
3. **Breaking changes require a migration.** If a server change would cause existing notes to render incorrectly or lose data, a vault schema migration must be written and shipped alongside it.
4. **Migrations are opt-in for the user.** The server logs migration activity and never silently modifies note content without bumping `schema_version`.
5. **Migrations are idempotent.** Re-running a migration against an already-migrated vault must be safe.

### Vault schema versioning

The server tracks the vault's schema version in `.mdkb/vault.toml`:

```toml
schema_version = 1
```

On every startup the server reads this file and runs any pending migrations in order, then updates the recorded version. Vaults created before v0.0.3 (which did not have this file) are treated as schema v0 and automatically stamped to v1 on first run (no content changes required).

### What constitutes a breaking change

| Change | Breaking? | Action required |
|---|---|---|
| Adding a new optional frontmatter field | No | None |
| Adding a new markdown extension | No | None |
| Adding a new config option with a safe default | No | None |
| Renaming an existing frontmatter field | Yes | Migration to rewrite affected notes; support old name as alias during transition |
| Changing journal path format | Yes | Migration to move existing journal entries |
| Changing wiki-link resolution semantics | Yes | Document transition; migration if old links would silently break |
| Changing `.mdkb/` metadata format | Internal only | Migration; no note content affected |

---

## Markdown Support

- **Flavor:** CommonMark + GitHub Flavored Markdown (GFM)
  - Tables, task lists, strikethrough, fenced code blocks
- **Wiki-links:** `[[Note Name]]` and `[[Note|Label]]` — resolves by filename, then frontmatter `title`
- **YAML frontmatter:** supported; recommended fields: `title`, `tags`, `created`, `updated`
- **Attachments (future):** images synced alongside parent notes

## Document Organization

No enforced structure. Defaults seeded on first Docker run:

```
vault/
  Welcome.md
  Getting-Started.md
  journal/
    YYYY/
      Mon/              # Jan, Feb, Mar …
        MM-DD-YYYY.md   # daily journal entries (auto-created on login)
  assets/
  .mdkb/                # tool metadata — not synced to clients
    sync/
```

---

## Milestones

| Milestone | Scope | Status |
|---|---|---|
| M0 — Foundations | Repo setup, Docker scaffold, basic API | ✅ Complete |
| M1 — Web UI | Read/edit markdown, folder tree, search, journal, Docker | ✅ Complete |
| M2 — Sync Protocol | Delta sync, version tracking, conflict detection | 🔲 Next |
| M3 — Desktop Client (Sync) | Background sync agent, offline queue, auto-reconnect | 🔲 Planned |
| M4 — Desktop Client (Editor) | Offline viewer + editor | 🔲 Planned |

---

## Feature Backlog (Post-v1)

Requests collected from early users — not yet scheduled for implementation:

| Feature | Notes |
|---|---|
| Full theming | Custom color schemes beyond accent color; CSS variable overrides; theme gallery |
| MermaidJS diagrams | Render `mermaid` fenced code blocks as flowcharts, sequence diagrams, etc. |
| MARP slide decks | Render MARP-formatted markdown as presentation slides in-browser |
| Focus mode | Distraction-free writing view: hide sidebar and chrome; toggled via keyboard shortcut |
| Auto-continue lists | When pressing Enter inside a list item, automatically insert the next list marker (`-`, `1.`, `- [ ]`) |
| Version update notification | Check for new releases and show a badge/prompt when a newer version is available |
| Configure page | In-app UI to edit `config.toml` fields (accent color, theme, vault path, API key rotation) with field descriptions and live preview |
| Image rendering | Render images stored in the vault directory; serve vault assets via `/api/files/*`; resolve relative paths in `![alt](path)` syntax |
| Tag browser | Filter and browse notes by tag; sidebar panel or search integration |
| Vault symlinks | Symlink files or directories from outside the vault into it (e.g. `vault/hash/ -> project docs`); requires WalkDir to preserve symlink-relative paths while reading through to target content |
| Vim keybindings | Optional vim modal editing (normal/insert/visual) in the editor pane; controlled via `vim_mode` config flag |

---

## Resolved Decisions

| Question | Decision |
|---|---|
| API key setup UX | Env var + `config.toml`; env vars sufficient for Docker with no config file |
| Desktop sync scope (v1) | Full vault sync only; folder selection deferred |
| Attachment sync | Auto-include linked attachments when syncing a note |
| Journal path format | `journal/YYYY/Mon/MM-DD-YYYY.md` |
| Empty journal cleanup | Auto-deleted when navigating away if content is only the heading |

---

## Revision History

| Version | Date | Notes |
|---|---|---|
| 0.1 | 2026-03-15 | Initial draft |
| 0.2 | 2026-03-15 | Tech stack decided |
| 0.3 | 2026-03-15 | Storage, auth, port, markdown flavor resolved |
| 0.4 | 2026-03-15 | All open questions closed; AGENTS.md added |
| 0.5 | 2026-03-15 | M0 and M1 complete; feature backlog added |
| 0.6 | 2026-03-16 | v0.0.3 RC: YAML frontmatter, journal button, hidden files config, line numbers, floating mode panel, search ranking, save dot, light mode contrast; backlog updated |
| 0.7 | 2026-03-16 | Note Format Specification and Backwards Compatibility Policy added; vault schema versioning and migration runner added (schema v1); spell-check config added |
