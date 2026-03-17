# Manual Testing Checklist — #ash

Use this checklist before tagging a release. Work through each section in order;
a fresh Docker dev environment is preferred so you start from a clean vault.

Mark each item ✅ pass, ❌ fail (with a note), or ⏭ skipped (with a reason).

---

## 0 — Setup

- [ ] `make dev` starts server on `:3535` and UI dev server on `:5173` without errors
- [ ] Opening `http://localhost:5173` shows the login screen
- [ ] Entering a wrong API key shows an error (does not proceed)
- [ ] Entering the correct API key logs in and opens today's journal entry

---

## 1 — Authentication

- [ ] API key is remembered after page reload (no re-login required)
- [ ] Logout button clears the key and returns to the login screen
- [ ] After logout, reloading the page shows the login screen (key not cached)

---

## 2 — File tree & navigation

- [ ] File tree shows all `.md` files and folders from the vault
- [ ] Clicking a file opens it in the editor
- [ ] Nested folders expand and collapse correctly
- [ ] `journal/` folder appears first in the tree regardless of alphabetical order
- [ ] Files inside folders sort alphabetically within their folder
- [ ] Hidden files (dotfiles) are **not** visible by default
- [ ] `⌘B` / `Ctrl+B` toggles the sidebar on desktop
- [ ] Sidebar collapse arrow hides the sidebar; the expand float button brings it back
- [ ] On mobile (≤640px), the sidebar is a drawer that opens via the hamburger float and closes on file select

---

## 3 — Creating & deleting files

- [ ] `⌘N` / `Ctrl+N` opens the new note modal
- [ ] Creating `note.md` creates the file and opens it
- [ ] Creating `folder/note.md` creates the folder and file, both appear in tree
- [ ] Attempting a path with `..` is rejected by the modal
- [ ] Hovering a file in the tree reveals a delete button
- [ ] Deleting a file removes it from the tree; if it was open, the editor clears
- [ ] Hovering a folder reveals a delete button; deleting a folder removes all contents
- [ ] Deleting the currently open file clears the editor

---

## 4 — Renaming

- [ ] Hovering a file reveals a rename button (pencil icon)
- [ ] Clicking rename shows an inline edit field pre-filled with the current name
- [ ] Renaming a file updates the tree and re-opens the file at the new path
- [ ] Renaming a folder updates the tree; the open file (if inside) reopens at the new path
- [ ] Pressing Escape cancels a rename without changing anything
- [ ] Renaming to an existing path — verify behavior is handled gracefully (no silent data loss)

---

## 5 — Editor

- [ ] Typing in the editor updates the preview pane immediately (split mode)
- [ ] Editor and preview scroll in sync when in split mode
- [ ] `⌘S` / `Ctrl+S` saves immediately; save dot briefly shows chartreuse
- [ ] Auto-save fires ~1.5s after the last keystroke (save dot pulses amber then goes chartreuse)
- [ ] Save dot stays amber while there are unsaved changes
- [ ] Save dot turns red if a save fails (simulate by stopping the server briefly)

**Auto-continue lists:**
- [ ] Pressing Enter at the end of `- item` inserts `- ` on the next line
- [ ] Pressing Enter at the end of `* item` inserts `* `
- [ ] Pressing Enter at the end of `1. item` inserts `2. ` (incrementing number)
- [ ] Pressing Enter at the end of `- [ ] task` inserts `- [ ] ` (unchecked)
- [ ] Pressing Enter at the end of `- [x] done` inserts `- [ ] ` (new item unchecked)
- [ ] Pressing Enter on an empty list item (`- `) exits the list (removes the marker)

---

## 6 — Preview / viewer

- [ ] Headings, paragraphs, bold, italic render correctly
- [ ] Fenced code blocks render with syntax highlighting
- [ ] `- [ ]` and `- [x]` render as visual checkboxes
- [ ] Clicking a checkbox in preview toggles `[ ]` ↔ `[x]` in the raw editor and triggers auto-save
- [ ] Tables render with bordered cells
- [ ] Blockquotes render with the accent color left border
- [ ] Horizontal rules (`---`) render as a thin line
- [ ] `[[Note Name]]` wiki-link renders as a styled link; clicking it opens that note
- [ ] `[[Note Name|Label]]` wiki-link shows "Label" as the link text
- [ ] A wiki-link to a non-existent note does nothing (no crash)

**Frontmatter:**
- [ ] A note with `tags: [foo, bar]` shows tag chips above the body
- [ ] A note with `aliases: my-alias` shows aliases in the properties panel
- [ ] Arbitrary frontmatter fields (e.g. `author: Jane`) appear as key-value rows
- [ ] A note with no frontmatter shows no properties panel

**File metadata footer:**
- [ ] Created date appears at the bottom of the note
- [ ] Modified date updates after saving

---

## 7 — Images

- [ ] An image referenced as `![alt](assets/photo.jpg)` renders in the preview
- [ ] Image paths are resolved relative to the note's directory
- [ ] An image in a subdirectory (`folder/assets/img.png`) resolves correctly from a note in `folder/`
- [ ] A non-existent image path shows a broken image icon (no crash)
- [ ] After a page reload, images continue to load (session token refreshed on login)

---

## 8 — Search

- [ ] Clicking the search bar or focusing it shows a search field
- [ ] Typing a query returns results within ~300ms
- [ ] Filename matches appear before content matches in results
- [ ] Search is case-insensitive
- [ ] Each result shows a file path and a snippet with the matching line
- [ ] Clicking a result opens the file
- [ ] An empty query returns no results
- [ ] A query with no matches shows an empty list

---

## 9 — Journal

- [ ] Calendar button in the sidebar opens today's journal entry (`journal/YYYY/Mon/MM-DD-YYYY.md`)
- [ ] If today's entry doesn't exist, it is created with a heading `# MM-DD-YYYY`
- [ ] If today's entry already exists, it is opened without creating a duplicate
- [ ] Navigating away from a blank journal entry (only the auto-heading, no other content) deletes it silently
- [ ] Navigating away from a journal entry with content does **not** delete it

---

## 10 — Theme & appearance

- [ ] Theme button in the header cycles Light → Dark → System → Light
- [ ] The header icon updates to match the active theme (sun / moon / monitor)
- [ ] System theme follows the OS dark mode preference when set to "System"
- [ ] Accent color is applied throughout (links, save dot, tag chips, checkboxes)
- [ ] Light mode: accent color is visibly darkened for legibility on white backgrounds
- [ ] Dark mode: accent color at full brightness

---

## 11 — Settings panel

**Opening:**
- [ ] Gear icon in the sidebar footer opens the settings panel
- [ ] `⌘,` / `Ctrl+,` also opens the settings panel
- [ ] Clicking outside the panel (backdrop) closes it
- [ ] `Escape` closes the panel

**Accent color:**
- [ ] Color picker updates the accent color live as you drag
- [ ] Hex text input accepts a valid hex color and applies it
- [ ] Color persists after closing and reopening the panel

**Theme:**
- [ ] Selecting Light/Dark/System applies the theme immediately
- [ ] The header theme icon updates to match the selection
- [ ] Theme selection persists after page reload

**Line numbers:**
- [ ] Toggling line numbers on shows a gutter in the editor immediately
- [ ] Toggling off hides the gutter immediately
- [ ] Setting persists after page reload

**Spell check:**
- [ ] Toggling spell check on enables browser spell-check underlines in the editor
- [ ] Setting persists after page reload

**Sync interval:**
- [ ] Changing the sync interval (e.g. from 10s to 5s) and committing updates the active timer without a page reload (verify by watching network requests in DevTools)
- [ ] Setting persists after page reload
- [ ] Value is clamped to a minimum of 1 second

**Large file threshold:**
- [ ] Changing the threshold saves without error
- [ ] Setting persists after page reload

---

## 12 — Sync & live updates

**Background poll:**
- [ ] Editing a file externally (e.g. with a text editor on disk) causes it to reload automatically in the browser within one poll cycle
- [ ] If the browser has unsaved edits when the external change arrives, the "file updated on server" banner appears
- [ ] Clicking "Reload" in the banner replaces the editor content with the server version
- [ ] Clicking ✕ in the banner dismisses it without reloading

**Focus sync:**
- [ ] Switching away to another app, making an external file change, then switching back triggers an immediate reload (within ~100ms)
- [ ] Switching between browser tabs (same behavior — tab focus triggers a sync check)
- [ ] Installing as a PWA: switching from the PWA back to the browser triggers sync in the browser tab (and vice versa)

**Open-file fast poll:**
- [ ] With a file open, external changes to that specific file are detected within ~2 seconds (faster than the vault poll interval)

---

## 13 — Editor mode switching

- [ ] Edit mode: only the textarea is visible, full width, max-width constrained
- [ ] Preview mode: only the rendered preview is visible
- [ ] Split mode (default): textarea on left, preview on right, scroll-synced
- [ ] Mode button panel floats on the right edge and doesn't obscure content
- [ ] Selected mode indicator highlights the active button

---

## 14 — PWA

- [ ] The app can be installed via "Add to Home Screen" / "Install app" in the browser
- [ ] Installed PWA loads without a server (shows cached app shell)
- [ ] Installed PWA connects to the server when online and loads notes normally
- [ ] PWA favicon and name appear correctly on the home screen

---

## 15 — Version & release links

- [ ] Version number appears in the sidebar footer (e.g. `v0.0.4`)
- [ ] Clicking the version number opens the GitHub release page for that version in a new tab
- [ ] GitHub icon opens the repository in a new tab
- [ ] Coffee icon opens the sponsorship page in a new tab

---

## 16 — Keyboard shortcuts summary

| Shortcut | Action |
|---|---|
| `⌘S` / `Ctrl+S` | Save current file |
| `⌘N` / `Ctrl+N` | New note modal |
| `⌘B` / `Ctrl+B` | Toggle sidebar |
| `⌘,` / `Ctrl+,` | Open settings |
| `Escape` | Close modal / settings panel |

- [ ] All shortcuts above work as described
- [ ] Shortcuts do not fire when focus is inside an `<input>` or modal (test: type `,` with settings open — should not re-open)

---

## 17 — Edge cases & error handling

- [ ] Opening a file with no content shows an empty editor (no crash)
- [ ] A very long file (>1000 lines) loads and scrolls without performance issues
- [ ] A file with special characters in the name (spaces, parentheses) opens correctly
- [ ] Navigating to a deep path (`a/b/c/d/note.md`) works; all ancestor folders expand
- [ ] Losing network connection mid-edit: save fails gracefully (red dot, no data loss)
- [ ] Reconnecting after a network interruption: next auto-save succeeds

---

*Last updated: v0.0.4*
