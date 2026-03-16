# Agents

This file documents how AI agents (Claude Code, etc.) should collaborate on this project.

## Project Identity

- **Name:** #ash (pronounced "hash")
- **Purpose:** Self-hosted, offline-first markdown knowledge base
- **Stack:** Rust/Axum (server), Tauri v2 + Svelte (desktop), Docker (deployment)
- **Requirements doc:** [REQUIREMENTS.md](REQUIREMENTS.md)
- **Container registry:** `ghcr.io/ryantiger658/hash`

## About the Human

- Knows Python and Ruby well; does not know Rust or JavaScript/TypeScript
- Comfortable reading code with explanation but should not be expected to write Rust or JS independently
- When explaining decisions, prefer plain language over jargon
- When writing code, prefer clarity and comments over terseness

## Collaboration Principles

- **Requirements first.** Before implementing anything non-trivial, verify the relevant section of REQUIREMENTS.md is up to date. If a decision is missing, surface it and document it before writing code.
- **Keep REQUIREMENTS.md current.** As decisions are made during implementation, update the requirements doc to reflect the final decision. The doc is the source of truth, not the code.
- **Iterate in small steps.** Prefer small, reviewable changes over large rewrites. Each session should leave the repo in a working (or clearly WIP-marked) state.
- **Explain tradeoffs.** When multiple approaches exist, briefly describe the options and recommend one with a rationale rather than silently choosing.
- **No unnecessary complexity.** This is a homelab tool for one user. Favor the simplest solution that meets the requirement.
- **Do not push without being asked.** Stage and commit, but wait for explicit instruction before `git push`.

## Conventions

### Commit Messages
- Imperative mood, present tense: `Add folder tree API` not `Added folder tree API`
- Reference the milestone if relevant: `[M1] Add markdown render endpoint`

### Versioning
- Semantic versioning (`MAJOR.MINOR.PATCH`) starting at `0.0.1`
- Git tags: `v0.0.1`, `v0.1.0`, etc.
- On a `v*` tag push, CI automatically builds and pushes the Docker image to GHCR

### Code Style
- Rust: follow `rustfmt` defaults; use `clippy` and resolve all warnings
- Svelte: Prettier defaults
- TOML config files: include comments explaining each field

### File Layout
```
hash/
  server/        # Rust server (Axum) — lib + bin split for testability
  desktop/       # Tauri v2 app
  ui/            # Shared Svelte frontend (server web UI + Tauri shell)
  docker/        # Dockerfile, docker-compose.yml, seed vault content
  dev/           # Local dev config and vault (gitignored except welcome files)
  REQUIREMENTS.md
  AGENTS.md
  Cargo.toml     # Workspace root (members: server)
```

### Branch Strategy
- `main` — stable, working state
- `milestone/M*` — active milestone work
- `feat/<name>` — individual features within a milestone

---

## Current Status

- [x] Requirements drafted and iterated (v0.5)
- [x] Repo structure scaffolded
- [x] M0: Foundations (Docker, config, vault API, CI/CD)
- [x] M1: Web UI (editor, search, journal, themes, tests)
- [ ] M2: Sync Protocol
- [ ] M3: Desktop Client (Sync)
- [ ] M4: Desktop Client (Editor)

---

## Decisions Log

| Decision | Outcome | Reason |
|---|---|---|
| Database vs file-based storage | File-based only | Transparent, no extra containers, easy backup |
| Electron vs Tauri | Tauri v2 | Smaller footprint, no bundled browser engine |
| Conflict resolution | Last-write-wins | No collaborative editing; keep it simple |
| Port | 3535 | ASCII 35 = `#`; 3535 = `##` markdown heading |
| Auth (v1) | Single API key in config.toml / env var | Simplest workable auth for single-user homelab |
| Attachment sync | Auto-include linked attachments | Syncing a note without its images is a broken experience |
| Desktop sync scope (v1) | Full vault only | Folder selection adds UI complexity; defer |
| Config loading | TOML file first; env vars if no file | Enables Docker-native deployment without a config file |
| Docker base image | Chainguard glibc-dynamic (distroless) | Near-zero CVE; runs as nonroot by default |
| Docker vault permissions | COPY seed files with `--chown=65532:65532` before VOLUME | Named volumes inherit image directory ownership; nonroot (uid 65532) must own `/vault` |
| Cargo structure | lib + bin split in `server/` | Integration tests in `tests/` can only import from a `[lib]` crate |
| Journal path | `journal/YYYY/Mon/MM-DD-YYYY.md` | Year/month-name folders keep the tree navigable; MM-DD-YYYY matches user preference |
| Empty journal cleanup | Auto-delete on navigate-away if only heading remains | Prevents accumulation of blank daily files |
| Default accent color | Chartreuse `#aaff00` | User preference |
| Editor mode buttons | Icons by default; `editor_labels = true` for text | Cleaner toolbar; accessible via config |
| Container registry | GHCR (`ghcr.io/ryantiger658/hash`) | Free for public repos; no separate login; push on `v*` tags |
