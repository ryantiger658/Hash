<script>
  import { onMount } from 'svelte'
  import { loadServerTheme, setTheme, getStoredTheme } from './lib/theme.js'
  import { hasApiKey, clearApiKey } from './lib/api.js'
  import {
    fileTree, files, selectedPath, fileContent, isDirty,
    loadVault, selectFile, createFile, deleteCurrentFile, saveCurrentFile,
  } from './stores/vault.js'
  import { get } from 'svelte/store'

  import LoginForm    from './lib/components/LoginForm.svelte'
  import FileTree     from './lib/components/FileTree.svelte'
  import Editor       from './lib/components/Editor.svelte'
  import SearchBar    from './lib/components/SearchBar.svelte'
  import NewItemModal from './lib/components/NewItemModal.svelte'

  // ── Auth state ───────────────────────────────────────────────────────────
  let authenticated = hasApiKey()

  function onLogin() {
    authenticated = true
    loadVault()
  }

  function logout() {
    clearApiKey()
    authenticated = false
    selectedPath.set(null)
    fileContent.set('')
  }

  // ── Theme ────────────────────────────────────────────────────────────────
  let currentTheme = getStoredTheme()
  const themeOrder = ['light', 'dark', 'system']
  const themeIcon  = { light: '☀️', dark: '🌙', system: '💻' }

  function cycleTheme() {
    const next = themeOrder[(themeOrder.indexOf(currentTheme) + 1) % themeOrder.length]
    currentTheme = next
    setTheme(next)
  }

  // ── New note modal ───────────────────────────────────────────────────────
  let showNewModal = false

  async function handleCreate(e) {
    await createFile(e.detail)
  }

  // ── File actions ─────────────────────────────────────────────────────────
  async function handleSelect(e) {
    const path = typeof e === 'string' ? e : e.detail
    await selectFile(path)
  }

  async function handleDelete() {
    if (!$selectedPath) return
    if (!confirm(`Delete "${$selectedPath}"? This cannot be undone.`)) return
    await deleteCurrentFile()
  }

  // Resolve a wiki-link: find best matching file by name or path
  async function handleWikiLink(e) {
    const target = e.detail.toLowerCase()
    const list = get(files)
    const found = list.find(f =>
      f.path.toLowerCase() === target + '.md' ||
      f.path.toLowerCase() === target ||
      f.path.toLowerCase().endsWith('/' + target + '.md') ||
      f.path.toLowerCase().endsWith('/' + target)
    )
    if (found) await selectFile(found.path)
  }

  // ── Keyboard shortcuts ───────────────────────────────────────────────────
  function onKeydown(e) {
    if ((e.metaKey || e.ctrlKey) && e.key === 's') {
      e.preventDefault()
      saveCurrentFile()
    }
    if ((e.metaKey || e.ctrlKey) && e.key === 'n') {
      e.preventDefault()
      showNewModal = true
    }
  }

  onMount(() => {
    loadServerTheme()
    if (authenticated) loadVault()
  })
</script>

<svelte:window on:keydown={onKeydown} />

{#if !authenticated}
  <LoginForm on:login={onLogin} />
{:else}
  <div class="app">
    <!-- ── Header ─────────────────────────────────────────────────────── -->
    <header>
      <span class="logo">#ash</span>

      <SearchBar on:select={handleSelect} />

      <nav class="header-actions">
        <button class="icon-btn" on:click={cycleTheme} title="Theme: {currentTheme}">
          {themeIcon[currentTheme]}
        </button>
        <button class="icon-btn muted" on:click={logout} title="Disconnect">⏏</button>
      </nav>
    </header>

    <!-- ── Body ──────────────────────────────────────────────────────── -->
    <div class="body">
      <!-- Sidebar -->
      <aside class="sidebar">
        <div class="sidebar-toolbar">
          <span class="sidebar-title">Notes</span>
          <button class="new-btn" on:click={() => (showNewModal = true)} title="New note (⌘N)">＋</button>
        </div>

        <div class="tree-scroll">
          <FileTree nodes={$fileTree} on:select={handleSelect} />
        </div>

        {#if $selectedPath}
          <div class="sidebar-footer">
            <button class="delete-btn" on:click={handleDelete} title="Delete current note">
              🗑 Delete
            </button>
          </div>
        {/if}
      </aside>

      <!-- Main content -->
      <main class="main">
        {#if $selectedPath}
          <Editor on:wikilink={handleWikiLink} />
        {:else}
          <div class="empty-state">
            <p class="empty-title">#ash</p>
            <p class="empty-sub">Select a note or <button class="link-btn" on:click={() => (showNewModal = true)}>create a new one</button>.</p>
            <p class="empty-hint">⌘S to save · ⌘N for a new note</p>
          </div>
        {/if}
      </main>
    </div>
  </div>

  <NewItemModal bind:show={showNewModal} on:create={handleCreate} />
{/if}

<style>
  /* ── CSS variables (dark default + light override) ───────────────────── */
  :global(:root) {
    --color-bg:         #0f1117;
    --color-surface:    #1a1d27;
    --color-border:     #2a2d3d;
    --color-text:       #e2e4ed;
    --color-text-muted: #8b8fa8;
    --color-accent:     #6366f1;
    --color-accent-dim: #6366f144;
  }

  :global([data-theme="light"]) {
    --color-bg:         #f5f6fa;
    --color-surface:    #ffffff;
    --color-border:     #dde0f0;
    --color-text:       #1a1d2e;
    --color-text-muted: #6b6e85;
    --color-accent-dim: #6366f122;
  }

  @media (prefers-color-scheme: light) {
    :global(:root:not([data-theme="dark"])) {
      --color-bg:         #f5f6fa;
      --color-surface:    #ffffff;
      --color-border:     #dde0f0;
      --color-text:       #1a1d2e;
      --color-text-muted: #6b6e85;
      --color-accent-dim: #6366f122;
    }
  }

  :global(*, *::before, *::after) { box-sizing: border-box; margin: 0; padding: 0; }
  :global(body) {
    font-family: system-ui, -apple-system, sans-serif;
    background: var(--color-bg);
    color: var(--color-text);
    height: 100vh;
    overflow: hidden;
  }

  /* ── Layout ──────────────────────────────────────────────────────────── */
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0 1rem;
    height: 48px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
    z-index: 10;
  }

  .logo {
    font-weight: 800;
    font-size: 1.05rem;
    color: var(--color-accent);
    white-space: nowrap;
  }

  .header-actions {
    display: flex;
    gap: 0.25rem;
    align-items: center;
    margin-left: auto;
  }

  .body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  /* ── Sidebar ─────────────────────────────────────────────────────────── */
  .sidebar {
    width: 220px;
    min-width: 160px;
    display: flex;
    flex-direction: column;
    background: var(--color-surface);
    border-right: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .sidebar-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.75rem 0.35rem;
    flex-shrink: 0;
  }

  .sidebar-title {
    font-size: 0.75rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--color-text-muted);
  }

  .tree-scroll {
    flex: 1;
    overflow-y: auto;
    padding: 0 0.3rem;
  }

  .sidebar-footer {
    padding: 0.5rem 0.75rem;
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  /* ── Main ────────────────────────────────────────────────────────────── */
  .main {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  /* ── Empty state ─────────────────────────────────────────────────────── */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 0.5rem;
    color: var(--color-text-muted);
  }

  .empty-title {
    font-size: 2.5rem;
    font-weight: 800;
    color: var(--color-accent);
    opacity: 0.3;
  }

  .empty-sub {
    font-size: 0.95rem;
  }

  .empty-hint {
    font-size: 0.78rem;
    opacity: 0.6;
  }

  /* ── Shared button styles ────────────────────────────────────────────── */
  .icon-btn {
    border: none;
    background: transparent;
    font-size: 1rem;
    cursor: pointer;
    padding: 0.25rem 0.4rem;
    border-radius: 5px;
    line-height: 1;
    transition: background 0.1s;
  }

  .icon-btn:hover {
    background: var(--color-border);
  }

  .icon-btn.muted {
    color: var(--color-text-muted);
    font-size: 0.9rem;
  }

  .new-btn {
    border: none;
    background: transparent;
    color: var(--color-accent);
    font-size: 1.2rem;
    font-weight: 300;
    cursor: pointer;
    line-height: 1;
    padding: 0 0.2rem;
    border-radius: 4px;
    transition: background 0.1s;
  }

  .new-btn:hover {
    background: var(--color-border);
  }

  .delete-btn {
    width: 100%;
    padding: 0.35rem;
    border: 1px solid var(--color-border);
    border-radius: 5px;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 0.8rem;
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s;
  }

  .delete-btn:hover {
    color: #f87171;
    border-color: #f87171;
  }

  .link-btn {
    border: none;
    background: transparent;
    color: var(--color-accent);
    font-size: inherit;
    cursor: pointer;
    padding: 0;
    text-decoration: underline;
  }
</style>
