<script>
  import { onMount } from 'svelte'
  import { loadServerTheme, setTheme, getStoredTheme } from './lib/theme.js'
  import { hasApiKey, clearApiKey } from './lib/api.js'
  import {
    fileTree, files, selectedPath, selectedFile, fileContent, isDirty,
    loadVault, selectFile, createFile, deleteFile, deleteFolder,
    saveCurrentFile, openTodayJournal,
  } from './stores/vault.js'
  import { version } from '../package.json'
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

  // ── Sidebar ───────────────────────────────────────────────────────────────
  let sidebarOpen = false       // mobile: drawer open
  let sidebarCollapsed = false  // desktop: panel collapsed

  // Hamburger: mobile drawer toggle + desktop expand (only appears when collapsed)
  function toggleMenu() {
    sidebarOpen = !sidebarOpen
    sidebarCollapsed = false  // on desktop: always expands
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
    sidebarOpen = false  // close drawer on mobile after selecting a file
  }

  async function handleDeleteFile(e) {
    await deleteFile(e.detail)
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
      <button class="icon-btn menu-btn" class:visible={sidebarCollapsed} on:click={toggleMenu} title="Toggle sidebar">
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
      <aside class="sidebar" class:open={sidebarOpen} class:collapsed={sidebarCollapsed}>
        <div class="sidebar-toolbar">
          <span class="sidebar-title">Notes</span>
          <div class="sidebar-toolbar-actions">
            <button class="new-btn" on:click={() => (showNewModal = true)} title="New note (⌘N)">
              <!-- Plus -->
              <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <line x1="8" y1="2" x2="8" y2="14"/>
                <line x1="2" y1="8" x2="14" y2="8"/>
              </svg>
            </button>
            <button class="collapse-btn" on:click={() => (sidebarCollapsed = true)} title="Collapse sidebar">
              <!-- Chevron left -->
              <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="10,3 5,8 10,13"/>
              </svg>
            </button>
          </div>
        </div>

        <div class="tree-scroll">
          <FileTree nodes={$fileTree} on:select={handleSelect} on:delete-folder={handleDeleteFolder} on:delete-file={handleDeleteFile} />
        </div>

        <div class="sidebar-footer">
          <a href="https://github.com/ryantiger658/Hash" target="_blank" rel="noopener noreferrer" class="sidebar-meta-link" title="View on GitHub">
            <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
            </svg>
            GitHub
          </a>
          <span class="app-version">v{version}</span>
        </div>
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

  .sidebar-toolbar-actions {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .collapse-btn {
    display: none;  /* mobile: hidden */
    align-items: center;
    justify-content: center;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    padding: 0 0.15rem;
    border-radius: 4px;
    transition: background 0.1s, color 0.1s;
  }

  .collapse-btn:hover {
    background: var(--color-border);
    color: var(--color-text);
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
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
  }

  .sidebar-meta-link {
    display: inline-flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.75rem;
    color: var(--color-text-muted);
    text-decoration: none;
    border-radius: 4px;
    padding: 0.1rem 0.25rem;
    transition: color 0.1s, background 0.1s;
  }

  .sidebar-meta-link:hover {
    color: var(--color-text);
    background: var(--color-border);
  }

  .app-version {
    font-size: 0.7rem;
    color: var(--color-text-muted);
    opacity: 0.6;
    font-variant-numeric: tabular-nums;
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
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: none;
    background: transparent;
    color: var(--color-text);
    font-size: 1rem;
    cursor: pointer;
    padding: 0.25rem 0.4rem;
    border-radius: 5px;
    line-height: 1;
    transition: background 0.1s;
    text-decoration: none;
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

  .link-btn {
    border: none;
    background: transparent;
    color: var(--color-accent);
    font-size: inherit;
    cursor: pointer;
    padding: 0;
    text-decoration: underline;
  }

  /* ── Hamburger: hidden on desktop by default; visible when collapsed ──── */
  .menu-btn { display: none; }
  .menu-btn.visible { display: inline-flex; }

  /* ── Desktop only ────────────────────────────────────────────────────── */
  @media (min-width: 641px) {
    .collapse-btn { display: inline-flex; }

    .sidebar.collapsed {
      width: 0;
      min-width: 0;
      overflow: hidden;
      border-right: none;
    }
  }

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
