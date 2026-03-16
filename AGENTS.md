# Agents

This file documents how AI agents (Claude Code, etc.) should collaborate on this project.

## Project Identity

- **Name:** #ash (pronounced "hash")
- **Purpose:** Self-hosted, offline-first markdown knowledge base
- **Stack:** Rust (server), Tauri + Svelte (desktop), Docker (deployment)
- **Requirements doc:** [REQUIREMENTS.md](REQUIREMENTS.md)

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

## Conventions

### Commit Messages
- Imperative mood, present tense: `Add folder tree API` not `Added folder tree API`
- Reference the milestone if relevant: `[M1] Add markdown render endpoint`

### Code Style
- Rust: follow `rustfmt` defaults; use `clippy` and resolve all warnings
- Svelte: Prettier defaults
- TOML config files: include comments explaining each field

### File Layout
```
hash/
  server/        # Rust server (Axum)
  desktop/       # Tauri app
  ui/            # Shared Svelte frontend (used by both server web UI and Tauri)
  docker/        # Dockerfile + docker-compose.yml
  docs/          # Architecture notes, API spec, user guide
  REQUIREMENTS.md
  AGENTS.md
  README.md
```

### Branch Strategy
- `main` — stable, working state
- `milestone/M0`, `milestone/M1`, etc. — active milestone work
- `feat/<name>` — individual features within a milestone

## Current Status

- [x] Requirements drafted and iterated (v0.3)
- [ ] Repo structure scaffolded
- [ ] M0: Foundations

## Decisions Log

Decisions that were discussed and resolved during development, for future context:

| Decision | Outcome | Reason |
|---|---|---|
| Database vs file-based storage | File-based only | Transparent, no extra containers, easy backup |
| Electron vs Tauri | Tauri | Smaller footprint, no bundled browser engine |
| Conflict resolution | Last-write-wins | No collaborative editing; keep it simple |
| Port | 3535 | ASCII 35 = `#`; 3535 = `##` markdown heading |
| Auth (v1) | Single API key in config.toml | Simplest workable auth for single-user homelab |
| Attachment sync | Auto-include linked attachments | Syncing a note without its images is a broken experience |
| Desktop sync scope (v1) | Full vault only | Folder selection adds UI complexity; defer |
