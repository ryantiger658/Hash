<script>
  import { onMount } from 'svelte'
  import { loadServerTheme, setTheme, getStoredTheme } from './lib/theme.js'
  import { hasApiKey, clearApiKey } from './lib/api.js'
  import {
    fileTree, files, selectedPath, selectedFile, fileContent, isDirty,
    loadVault, selectFile, createFile, deleteCurrentFile, deleteFolder,
    saveCurrentFile, openTodayJournal,
  } from './stores/vault.js'
  import { get } from 'svelte/store'

  import LoginForm    from './lib/components/LoginForm.svelte'
  import FileTree     from './lib/components/FileTree.svelte'
  import Editor       from './lib/components/Editor.svelte'
  import SearchBar    from './lib/components/SearchBar.svelte'
  import NewItemModal from './lib/components/NewItemModal.svelte'

  // ── Auth state ───────────────────────────────────────────────────────────
  let authenticated = hasApiKey()

  async function onLogin() {
    authenticated = true
    await loadVault()
    await openTodayJournal()
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

  function cycleTheme() {
    const next = themeOrder[(themeOrder.indexOf(currentTheme) + 1) % themeOrder.length]
    currentTheme = next
    setTheme(next)
  }

  // ── Sidebar (mobile drawer) ──────────────────────────────────────────────
  let sidebarOpen = false

  // ── New note modal ───────────────────────────────────────────────────────
  let showNewModal = false

  async function handleCreate(e) {
    await createFile(e.detail)
  }

  // ── File actions ─────────────────────────────────────────────────────────
  async function handleSelect(e) {
    const path = typeof e === 'string' ? e : e.detail
    await selectFile(path)
    sidebarOpen = false  // close drawer on mobile after selecting a file
  }

  async function handleDelete() {
    if (!$selectedPath) return
    if (!confirm(`Delete "${$selectedPath}"? This cannot be undone.`)) return
    await deleteCurrentFile()
  }

  async function handleDeleteFolder(e) {
    await deleteFolder(e.detail)
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

  onMount(async () => {
    loadServerTheme()
    if (authenticated) {
      await loadVault()
      await openTodayJournal()
    }
  })
</script>

<svelte:window on:keydown={onKeydown} />

{#if !authenticated}
  <LoginForm on:login={onLogin} />
{:else}
  <div class="app">
    <!-- ── Header ─────────────────────────────────────────────────────── -->
    <header>
      <button class="icon-btn menu-btn" on:click={() => (sidebarOpen = !sidebarOpen)} title="Menu">
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round">
          <line x1="2" y1="4" x2="14" y2="4"/>
          <line x1="2" y1="8" x2="14" y2="8"/>
          <line x1="2" y1="12" x2="14" y2="12"/>
        </svg>
      </button>

      <span class="logo">#</span>

      <SearchBar on:select={handleSelect} />

      <nav class="header-actions">
        <button class="icon-btn theme-btn" on:click={cycleTheme} title="Theme: {currentTheme}">
          {#if currentTheme === 'light'}
            <!-- Sun -->
            <svg width="15" height="15" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="8" cy="8" r="3"/>
              <line x1="8" y1="1" x2="8" y2="3"/>
              <line x1="8" y1="13" x2="8" y2="15"/>
              <line x1="1" y1="8" x2="3" y2="8"/>
              <line x1="13" y1="8" x2="15" y2="8"/>
              <line x1="3.05" y1="3.05" x2="4.46" y2="4.46"/>
              <line x1="11.54" y1="11.54" x2="12.95" y2="12.95"/>
              <line x1="12.95" y1="3.05" x2="11.54" y2="4.46"/>
              <line x1="4.46" y1="11.54" x2="3.05" y2="12.95"/>
            </svg>
          {:else if currentTheme === 'dark'}
            <!-- Moon -->
            <svg width="15" height="15" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M13 10A6 6 0 016 3a6 6 0 100 10 6 6 0 007-3z"/>
            </svg>
          {:else}
            <!-- Monitor -->
            <svg width="15" height="15" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <rect x="1" y="2" width="14" height="10" rx="1.5"/>
              <line x1="5" y1="14" x2="11" y2="14"/>
              <line x1="8" y1="12" x2="8" y2="14"/>
            </svg>
          {/if}
        </button>
        <button class="icon-btn muted" on:click={logout} title="Logout">⏏</button>
      </nav>
    </header>

    <!-- ── Body ──────────────────────────────────────────────────────── -->
    <div class="body">
      <!-- Mobile overlay backdrop -->
      {#if sidebarOpen}
        <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
        <div class="sidebar-backdrop" on:click={() => (sidebarOpen = false)}></div>
      {/if}

      <!-- Sidebar -->
      <aside class="sidebar" class:open={sidebarOpen}>
        <div class="sidebar-toolbar">
          <span class="sidebar-title">Notes</span>
          <button class="new-btn" on:click={() => (showNewModal = true)} title="New note (⌘N)">
            <!-- Plus -->
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
              <line x1="8" y1="2" x2="8" y2="14"/>
              <line x1="2" y1="8" x2="14" y2="8"/>
            </svg>
          </button>
        </div>

        <div class="tree-scroll">
          <FileTree nodes={$fileTree} on:select={handleSelect} on:delete-folder={handleDeleteFolder} />
        </div>

        {#if $selectedPath}
          <div class="sidebar-footer">
            <button class="delete-btn" on:click={handleDelete} title="Delete current note">
              <!-- Trash -->
              <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M2 4h12"/>
                <path d="M5 4V2h6v2"/>
                <path d="M3 4l1 10h8l1-10"/>
                <path d="M6.5 7v4M9.5 7v4"/>
              </svg>
              Delete
            </button>
          </div>
        {/if}
      </aside>

      <!-- Main content -->
      <main class="main">
        {#if $selectedPath}
          <Editor file={$selectedFile} on:wikilink={handleWikiLink} />
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
    --color-bg:         #000000;
    --color-surface:    #0d0d0d;
    --color-border:     #1f1f1f;
    --color-text:       #e2e4ed;
    --color-text-muted: #6b6e85;
    --color-accent:     #aaff00;
    --color-accent-dim: #aaff0044;
  }

  :global([data-theme="light"]) {
    --color-bg:         #f5f6fa;
    --color-surface:    #ffffff;
    --color-border:     #dde0f0;
    --color-text:       #1a1d2e;
    --color-text-muted: #6b6e85;
    --color-accent-dim: #aaff0033;
  }

  @media (prefers-color-scheme: light) {
    :global(:root:not([data-theme="dark"])) {
      --color-bg:         #f5f6fa;
      --color-surface:    #ffffff;
      --color-border:     #dde0f0;
      --color-text:       #1a1d2e;
      --color-text-muted: #6b6e85;
      --color-accent-dim: #aaff0033;
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

  /* ── Hamburger (hidden on desktop) ───────────────────────────────────── */
  .menu-btn { display: none; }

  /* ── Responsive ──────────────────────────────────────────────────────── */
  @media (max-width: 640px) {
    .menu-btn { display: flex; }

    .sidebar {
      position: fixed;
      top: 48px;
      left: 0;
      bottom: 0;
      z-index: 200;
      width: 280px;
      transform: translateX(-100%);
      transition: transform 0.22s ease;
      box-shadow: 4px 0 32px rgba(0, 0, 0, 0.6);
    }

    .sidebar.open {
      transform: translateX(0);
    }

    .sidebar-backdrop {
      position: fixed;
      inset: 48px 0 0 0;
      z-index: 199;
      background: rgba(0, 0, 0, 0.55);
    }

    /* Full-width main on mobile */
    .body {
      overflow: auto;
    }

    header {
      gap: 0.5rem;
      padding: 0 0.6rem;
    }

    .empty-hint {
      display: none;
    }
  }
</style>
