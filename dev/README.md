# dev/

Local development environment for #ash. This folder is **not deployed** — it holds your local config and test vault.

## Setup

```bash
make setup
```

This copies `server/config.example.toml` to `dev/config.toml` and creates the vault directory. Edit `dev/config.toml` before running the server.

## Contents

| Path | Tracked? | Purpose |
|---|---|---|
| `dev/config.toml` | No (gitignored) | Local server config with your API key |
| `dev/vault/` | Dir only | Test markdown vault — put your notes here |
