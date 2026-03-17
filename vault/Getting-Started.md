---
title: Getting Started
tags: [getting-started, guide]
created: 2026-03-15
updated: 2026-03-16
---

# Getting Started with #ash

This guide walks through the features of #ash. It's also a live demo of the markdown rendering — everything you see here is plain text underneath.

---

## Markdown basics

#ash renders **CommonMark** with **GitHub Flavored Markdown (GFM)** extensions.

### Text formatting

You can write **bold**, *italic*, ~~strikethrough~~, and `inline code`.

> Blockquotes work great for callouts and pull quotes.

### Headings

Use `#` through `######` for six levels of headings.

### Links

- External links: [#ash on GitHub](https://github.com/ryantiger658/Hash)
- Wiki-links to other notes: [[Welcome]]
- Wiki-links with display text: [[Welcome|Back to Welcome]]

---

## Lists

### Unordered

- First item
- Second item
  - Nested item
  - Another nested item
- Third item

### Ordered

1. Install #ash
2. Mount your vault
3. Start writing

### Task lists

- [x] Create your first note
- [ ] Explore the file tree
- [ ] Customize your accent color
- [ ] Set up your journal folder

---

## Code

Inline code: `const greeting = "hello, #ash"`

Fenced code block with syntax highlighting:

```javascript
async function syncVault(serverUrl, apiKey) {
  const snapshot = await fetch(`${serverUrl}/api/sync/snapshot`, {
    headers: { Authorization: `Bearer ${apiKey}` }
  }).then(r => r.json())

  console.log(`Vault has ${snapshot.files.length} files`)
}
```

```rust
// The #ash server is written in Rust
fn main() {
    println!("Starting #ash on port 3535");
}
```

---

## Tables

| Feature | Status |
|---|---|
| Read markdown | ✓ |
| Edit markdown | ✓ |
| Split editor | ✓ |
| Full-text search | ✓ |
| YAML frontmatter | ✓ |
| Wiki-links | ✓ |
| Daily journal | ✓ |
| Desktop sync | Coming soon |

---

## YAML Frontmatter

Notes can have optional YAML frontmatter between `---` markers at the top of the file:

```yaml
---
title: My Note
tags: [work, ideas]
aliases: [my note, work note]
created: 2026-03-16
---
```

Frontmatter is displayed as a properties panel above the note content in Preview mode. Tags render as colored chips; aliases let wiki-links resolve by alternate names.

**Supported fields:**
- `tags` / `tag` — array or single string; displayed as chips
- `aliases` / `alias` — alternate names for wiki-link resolution
- Any other key-value pair is shown in the properties panel

---

## Wiki-links

Connect notes with `[[Note Name]]` syntax. You can also use display text: `[[Note Name|Label]]`.

Wiki-links resolve in this order:
1. Frontmatter `aliases` (if the target matches a registered alias)
2. Exact vault path (`notes/my-note.md`)
3. Filename match anywhere in the vault

---

## Daily Journal

Click the **calendar icon** in the sidebar toolbar to open today's journal entry. If it doesn't exist yet, it's created automatically at:

```
journal/YYYY/Mon/MM-DD-YYYY.md
```

If you open today's journal and navigate away without writing anything, the empty entry is automatically deleted.

---

## Editor modes

The floating panel on the **right edge of the screen** switches between three modes:

| Mode | Description |
|---|---|
| **Edit** | Raw markdown editor only |
| **Split** | Editor and preview side by side (default) |
| **Preview** | Rendered view only |

The **status dot** at the bottom of the panel shows save state: amber (pulsing) when you have unsaved changes, chartreuse when fully saved.

Use `⌘S` (or `Ctrl+S`) to save manually at any time. Changes are also auto-saved 1.5 seconds after you stop typing.

---

## Search

Press the search bar in the header to search your vault. Results are ranked:

1. **Filename matches** — files whose path contains your query
2. **Content matches** — files whose content contains your query (shows a snippet)

---

## Organizing your vault

There are no rules — use whatever folder structure works for you:

**By topic**
```
Work/
  Projects/
  Meetings/
Personal/
  Ideas/
  Journal/
```

**By date (Zettelkasten-style)**
```
2026/
  03/
    2026-03-16 project kickoff.md
```

**Flat with tags**
```
note-on-rust.md        ← tags: [dev, rust]
book-notes-dune.md     ← tags: [reading]
```

---

## Keyboard shortcuts

| Shortcut | Action |
|---|---|
| `⌘S` / `Ctrl+S` | Save current file |
| `⌘N` / `Ctrl+N` | New note |
| `⌘B` / `Ctrl+B` | Toggle sidebar |
| `⌘,` / `Ctrl+,` | Open Settings panel |
| `Escape` | Close modal or Settings panel |

**In the editor:**

| Shortcut | Action |
|---|---|
| `Enter` after `- item` | Continue list with `- ` |
| `Enter` after `* item` | Continue list with `* ` |
| `Enter` after `1. item` | Continue with `2.` (auto-increments) |
| `Enter` after `- [ ] task` | Continue with `- [ ] ` (new unchecked item) |
| `Enter` on empty list item | Exit the list |

> **Tip:** Click any checkbox in the Preview pane to toggle it directly — the source updates automatically.

---

## Theming

Click the **theme icon** in the header to cycle between **Light**, **Dark**, and **System**.

Open the **Settings panel** (`⌘,` / `Ctrl+,`) to change the accent color live without a restart. Settings are saved automatically.

You can also set defaults in `config.toml` on the server:

```toml
[ui]
secondary_color = "#e11d48"   # rose
default_theme = "dark"
```

Some color ideas:

| Name | Hex |
|---|---|
| Chartreuse (default) | `#aaff00` |
| Violet | `#7c3aed` |
| Rose | `#e11d48` |
| Emerald | `#10b981` |
| Amber | `#f59e0b` |
| Sky | `#0ea5e9` |
| Indigo | `#6366f1` |

> **Tip:** Any hex color in backtick code (`#aaff00`) renders with a color preview chip.

---

> **Tip:** Delete this file and `Welcome.md` once you're comfortable — they're just here to get you started.
