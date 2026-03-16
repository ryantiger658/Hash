---
title: Getting Started
tags: [getting-started, guide]
created: 2026-03-15
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

Use `#` through `######` for six levels of headings. The heading hierarchy is reflected in the document outline.

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
- [x] Explore the file tree
- [ ] Set up the desktop client
- [ ] Configure your SMB vault
- [ ] Customize your accent color

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

| Feature | Web UI | Desktop |
|---|---|---|
| Read markdown | ✓ | ✓ |
| Edit markdown | ✓ | ✓ |
| Works offline | — | ✓ |
| Auto-sync | ✓ | ✓ |
| Full-text search | ✓ | ✓ |

---

## Frontmatter

Every note can have optional YAML frontmatter at the top (between `---` markers):

```yaml
---
title: My Note
tags: [work, ideas]
created: 2026-03-15
---
```

Frontmatter is used for search, tagging, and wiki-link resolution by title.

---

## Organizing your vault

There are no rules — use whatever folder structure works for you. A few popular approaches:

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
    2026-03-15 project kickoff.md
```

**Flat with tags**
```
note-on-rust.md        ← tags: [dev, rust]
book-notes-dune.md     ← tags: [reading]
```

---

## Desktop client

The #ash desktop app keeps a local copy of your vault and syncs it with the server.

1. Download the client for your platform from the [Releases page](https://github.com/ryantiger658/Hash/releases)
2. Open the app and enter your server URL: `http://your-server:3535`
3. Enter your API key from `config.toml`
4. Choose a local folder to sync into

The client syncs automatically and queues any changes you make while offline.

---

## Theming

Click the theme toggle in the top-right corner to switch between **light**, **dark**, and **system** (follows your OS setting).

To change the accent color, edit `config.toml` on the server:

```toml
[ui]
secondary_color = "#e11d48"   # rose
default_theme = "dark"
```

Then restart the server. Some color ideas:

| Name | Hex |
|---|---|
| Indigo (default) | `#6366f1` |
| Violet | `#7c3aed` |
| Rose | `#e11d48` |
| Emerald | `#10b981` |
| Amber | `#f59e0b` |
| Sky | `#0ea5e9` |

---

> **Tip:** Delete this file and `Welcome.md` once you're comfortable — they're just here to get you started.
