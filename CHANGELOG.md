# Changelog

All notable changes to #ash are documented here. Versions follow [Semantic Versioning](https://semver.org/).

---

## [v0.0.4] — *In progress*

### New Features

- **Live sync** — The editor automatically detects when a file has been changed externally (by another browser, device, or directly on disk) and reloads it silently. If you have unsaved edits when an external change arrives, a banner appears so you can choose to reload or keep your work.
- **Focus sync** — Switching back to the browser tab or PWA window immediately triggers a sync check, so you always see the latest version of your notes without waiting for the next poll cycle.
- **Image rendering** — Images stored in your vault now display correctly in the preview pane. Reference them with standard markdown syntax: `![alt](relative/path.png)`.
- **Inline file rename** — Hover over any file or folder in the sidebar to reveal a rename button. Rename in place without leaving the editor.
- **Folder delete** — Delete entire folders and their contents from the sidebar with a single click.
- **Auto-continue lists** — Pressing Enter inside a bulleted or numbered list automatically inserts the next list marker. Press Enter again on an empty item to exit the list.
- **In-app settings panel** — A settings panel (gear icon in the sidebar footer) lets you change theme, accent color, line numbers, spell check, sync interval, and large file threshold without restarting the server. Changes apply instantly and persist across reloads.
- **Installable PWA** — #ash can be installed as a Progressive Web App on desktop and mobile. Use your browser's "Install" option to add it to your home screen or dock.
- **Sync protocol (M2)** — The server now tracks per-file sync metadata in `.mdkb/sync/`. The `/api/sync/snapshot` and `/api/sync/push` endpoints support conflict detection for future desktop clients.

### Improvements

- Accent color in the settings panel shows a live preview as you drag the color picker — no save button needed.
- Theme changes in the settings panel update the header icon immediately.
- Large files (default: over 512 KB) use a fast mtime+size check instead of a full SHA-256 hash, reducing CPU usage during sync polling.
- The currently open file is checked for changes every 2 seconds using a lightweight single-file endpoint, keeping the editor responsive to external edits.
- Sync poll interval and large file threshold are now configurable from the settings panel.

---

## [v0.0.3] — 2026-03-15

### New Features

- **YAML frontmatter** — Notes with a YAML block at the top (`---` delimiters) display properties, tags, and aliases in a panel above the note body. Compatible with Obsidian frontmatter.
- **Tag chips** — Tags declared in frontmatter are displayed as styled chips in the properties panel.
- **Wiki-link aliases** — `aliases` declared in frontmatter are used to resolve `[[wiki-links]]` — link to a note by any of its alternate names.
- **Journal button** — A calendar icon in the sidebar toolbar opens today's journal entry directly (`journal/YYYY/Mon/MM-DD-YYYY.md`). The entry is created automatically if it doesn't exist, and deleted silently if you navigate away without adding content.
- **Line numbers** — Optionally display line numbers in the editor pane. Toggle via server config (`line_numbers = true`) or the in-app settings panel.
- **Spell check** — Optionally enable browser spell-check underlining in the editor. Toggle via server config (`spell_check = true`) or the in-app settings panel.
- **Hidden files** — Dotfiles and hidden directories are hidden from the file tree by default. Set `show_hidden_files = true` in your config to reveal them.
- **Save status dot** — A small indicator dot in the floating mode panel shows your save state at a glance: amber (unsaved changes), chartreuse (saved), red (save failed).
- **Search ranking** — Full-text search returns filename matches before content matches, so the most relevant results appear first.
- **Vault schema versioning** — The server tracks a schema version in `.mdkb/vault.toml` and runs automatic migrations on startup. Existing vaults from v0.0.1 and v0.0.2 are migrated automatically with no action required.

### Improvements

- Floating Edit / Split / Preview mode panel moved to the right edge of the editor area.
- Improved contrast in light mode — accent color is automatically darkened for legibility on light backgrounds.
- File metadata footer (created date, last modified) rendered below each note in preview.

---

## [v0.0.2] — 2026-03-15

### New Features

- **Syntax highlighting** — Code blocks in the preview pane are syntax-highlighted.
- **Task lists** — `- [ ]` and `- [x]` checkboxes render as interactive-looking checkboxes in the preview.
- **Hack font** — The editor pane uses the Hack monospace font for comfortable long-form writing.
- **Sidebar collapse** — On desktop, the sidebar can be collapsed to give the editor more room. Use the arrow button or `⌘B` / `Ctrl+B`.
- **Delete on hover** — A delete button appears when hovering over a file in the sidebar, keeping the interface clean during normal use.
- **Sidebar footer** — Version number and project links appear in a tidy footer at the bottom of the sidebar.

### Improvements

- Editor and preview panes scroll in sync when in split mode.
- General UI polish across light and dark themes.

---

## [v0.0.1] — 2026-03-15

Initial release.

### Features

- **Markdown editor** — Split view with a raw editor on the left and a live preview on the right. Switch between Edit, Split, and Preview modes.
- **Folder tree** — Browse your vault in a collapsible sidebar. Empty folders are visible.
- **Full-text search** — Search across all `.md` files by filename or content. Results include a path, snippet, and line number.
- **Auto-save** — Changes are saved automatically 1.5 seconds after you stop typing. Manual save with `⌘S` / `Ctrl+S`.
- **New note modal** — Create notes at any path, including nested folders (`folder/subfolder/note.md`). Trigger with `⌘N` / `Ctrl+N`.
- **Wiki-links** — Link between notes with `[[Note Name]]` or `[[Note Name|Label]]`. Resolved by filename.
- **Light / dark / system theme** — Follows your OS preference by default; override with the theme button in the header.
- **Accent color** — Configurable via `config.toml` or `HASH_SECONDARY_COLOR`. Default: chartreuse (`#aaff00`).
- **Login gate** — API key authentication. Key is stored in browser `localStorage` so you only log in once per browser.
- **Docker deployment** — Single image (`ghcr.io/ryantiger658/hash`). Mount your vault directory and set two environment variables to get started.
- **Welcome content** — A named Docker volume seeds a `Welcome.md` and `Getting-Started.md` on first run.
- **Path traversal protection** — All file operations are sandboxed to the vault root.
