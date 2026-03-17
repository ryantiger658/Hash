<script>
  import { onMount } from 'svelte'
  import { loadServerTheme, setTheme, activeTheme, refreshImageToken } from './lib/theme.js'
  import { api, hasApiKey, clearApiKey } from './lib/api.js'
  import {
    fileTree, files, selectedPath, selectedFile, fileContent,
    isDirty, saveStatus, aliasMap, remoteChangeAvailable, pollIntervalSecs,
    loadVault, selectFile, createFile, deleteFile, deleteFolder, renameItem,
    saveCurrentFile, openTodayJournal, uploadFiles,
    startPolling, stopPolling, pollVault, acceptRemoteChange,
    startOpenFilePoll, stopOpenFilePoll,
  } from './stores/vault.js'
  import { version } from '../package.json'
  import { get } from 'svelte/store'

  import LoginForm    from './lib/components/LoginForm.svelte'
  import FileTree     from './lib/components/FileTree.svelte'
  import Editor       from './lib/components/Editor.svelte'
  import AssetViewer  from './lib/components/AssetViewer.svelte'
  import SearchBar    from './lib/components/SearchBar.svelte'
  import NewItemModal    from './lib/components/NewItemModal.svelte'
  import SettingsPanel   from './lib/components/SettingsPanel.svelte'

  // ── Auth state ───────────────────────────────────────────────────────────
  let authenticated = hasApiKey()

  async function onLogin() {
    authenticated = true
    refreshImageToken()
    const cfg = await loadServerTheme()
    const pollMs = (cfg?.poll_interval_secs ?? 10) * 1000
    pollIntervalSecs.set(cfg?.poll_interval_secs ?? 10)
    await loadVault()
    await openTodayJournal()
    startPolling(pollMs)
  }

  function logout() {
    stopPolling()
    stopOpenFilePoll()
    clearApiKey()
    authenticated = false
    selectedPath.set(null)
    fileContent.set('')
  }

  function onPollIntervalChange(e) {
    stopPolling()
    startPolling(e.detail.secs * 1000)
  }

  // ── Theme ────────────────────────────────────────────────────────────────
  const themeOrder = ['light', 'dark', 'system']

  function cycleTheme() {
    const next = themeOrder[(themeOrder.indexOf($activeTheme) + 1) % themeOrder.length]
    setTheme(next)
    // Persist to server so the setting survives page reload.
    if (authenticated) api.postUiConfig({ default_theme: next }).catch(() => {})
  }

  // ── Editor mode ──────────────────────────────────────────────────────────
  let editorMode = 'split'
  const saveLabels = { idle: '', saving: 'Saving…', saved: '✓ Saved', error: 'Save failed' }

  // ── Sidebar ───────────────────────────────────────────────────────────────
  let sidebarOpen = false       // mobile: drawer open
  let sidebarCollapsed = false  // desktop: panel collapsed

  // Compute which folder paths need to be open to reveal the selected file
  $: openPaths = (() => {
    if (!$selectedPath) return new Set()
    const parts = $selectedPath.split('/')
    const s = new Set()
    for (let i = 1; i < parts.length; i++) s.add(parts.slice(0, i).join('/'))
    return s
  })()

  // ── Modals ───────────────────────────────────────────────────────────────
  let showNewModal    = false
  let showSettings = false

  // ── Upload ────────────────────────────────────────────────────────────────
  let uploadInput

  function triggerUpload() { uploadInput?.click() }

  async function handleUpload(e) {
    const files = e.target.files
    if (!files?.length) return
    // Target the folder of the currently open file, or vault root
    const folder = $selectedPath?.includes('/')
      ? $selectedPath.slice(0, $selectedPath.lastIndexOf('/'))
      : ''
    await uploadFiles(files, folder)
    e.target.value = ''   // reset so re-uploading the same file triggers change
  }

  // True when the selected file is a non-markdown asset
  $: isAsset = $selectedPath ? !$selectedPath.endsWith('.md') : false

  function openSettings() {
    showSettings = true
  }

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

  async function handleRename(e) {
    await renameItem(e.detail.from, e.detail.to)
  }

  // Resolve a wiki-link: check aliases first, then match by path/name
  async function handleWikiLink(e) {
    const target = e.detail.toLowerCase()

    // 1. Check alias cache (built up as files are opened)
    const aMap = get(aliasMap)
    if (aMap.has(target)) {
      await selectFile(aMap.get(target))
      return
    }

    // 2. Match by path or filename
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
    if ((e.metaKey || e.ctrlKey) && e.key === 'b') {
      e.preventDefault()
      if (window.innerWidth <= 640) {
        sidebarOpen = !sidebarOpen
      } else {
        sidebarCollapsed = !sidebarCollapsed
      }
    }
    if ((e.metaKey || e.ctrlKey) && e.key === ',') {
      e.preventDefault()
      showSettings = true
    }
  }

  // Restart the open-file fast poll whenever the selected file changes.
  $: {
    stopOpenFilePoll()
    if ($selectedPath && authenticated) startOpenFilePoll()
  }

  onMount(() => {
    loadServerTheme().then(cfg => {
      const pollMs = (cfg?.poll_interval_secs ?? 10) * 1000
      pollIntervalSecs.set(cfg?.poll_interval_secs ?? 10)
      if (authenticated) {
        refreshImageToken()
        loadVault().then(openTodayJournal).then(() => startPolling(pollMs))
      }
    })

    // Poll when the page becomes active again. Two complementary events:
    //   visibilitychange → fires when switching browser tabs (tab goes hidden → visible)
    //   window focus     → fires when the OS window regains focus (e.g. PWA → browser, or
    //                       browser minimised then restored)
    //
    // Use hasApiKey() at call-time rather than the closure-captured `authenticated`
    // variable — Svelte's reactive invalidation can mean the closure sees a stale value.
    // Debounce so both events firing together (e.g. restoring a minimised window) only
    // triggers one poll.
    let focusPollTimer = null
    function scheduleFocusPoll() {
      if (!hasApiKey()) return
      clearTimeout(focusPollTimer)
      focusPollTimer = setTimeout(pollVault, 100)
    }

    function onVisibilityChange() {
      if (document.visibilityState === 'visible') scheduleFocusPoll()
    }

    document.addEventListener('visibilitychange', onVisibilityChange)
    window.addEventListener('focus', scheduleFocusPoll)

    return () => {
      document.removeEventListener('visibilitychange', onVisibilityChange)
      window.removeEventListener('focus', scheduleFocusPoll)
      clearTimeout(focusPollTimer)
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
      <span class="logo">#</span>

      <SearchBar on:select={handleSelect} />

      <nav class="header-actions">
        <button class="icon-btn theme-btn" on:click={cycleTheme} title="Theme: {$activeTheme}">
          {#if $activeTheme === 'light'}
            <!-- Sun -->
            <svg width="15" height="15" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <circle cx="8" cy="8" r="2.5"/>
              <line x1="8" y1="1" x2="8" y2="3"/>
              <line x1="8" y1="13" x2="8" y2="15"/>
              <line x1="1" y1="8" x2="3" y2="8"/>
              <line x1="13" y1="8" x2="15" y2="8"/>
              <line x1="3.05" y1="3.05" x2="4.46" y2="4.46"/>
              <line x1="11.54" y1="11.54" x2="12.95" y2="12.95"/>
              <line x1="12.95" y1="3.05" x2="11.54" y2="4.46"/>
              <line x1="4.46" y1="11.54" x2="3.05" y2="12.95"/>
            </svg>
          {:else if $activeTheme === 'dark'}
            <!-- Moon — filled crescent -->
            <svg width="15" height="15" viewBox="0 0 16 16" fill="currentColor" stroke="none">
              <path d="M6 .278a.768.768 0 0 1 .08.858 7.208 7.208 0 0 0-.878 3.46c0 4.021 3.278 7.277 7.318 7.277.527 0 1.04-.055 1.533-.16a.787.787 0 0 1 .81.316.733.733 0 0 1-.031.893A8.349 8.349 0 0 1 8.344 16C3.734 16 0 12.286 0 7.71 0 4.266 2.114 1.312 5.124.06A.752.752 0 0 1 6 .278z"/>
            </svg>
          {:else}
            <!-- Monitor -->
            <svg width="15" height="15" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
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

      <!-- Floating expand button: desktop when collapsed, mobile when drawer closed -->
      {#if sidebarCollapsed}
        <button class="sidebar-expand-float desktop-float" on:click={() => (sidebarCollapsed = false)} title="Expand sidebar">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="6,3 11,8 6,13"/>
          </svg>
        </button>
      {/if}
      {#if !sidebarOpen}
        <button class="sidebar-expand-float mobile-float" on:click={() => (sidebarOpen = true)} title="Open sidebar">
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="6,3 11,8 6,13"/>
          </svg>
        </button>
      {/if}

      <!-- Sidebar -->
      <aside class="sidebar" class:open={sidebarOpen} class:collapsed={sidebarCollapsed}>
        <div class="sidebar-toolbar">
          <button class="collapse-btn" on:click={() => (sidebarCollapsed = !sidebarCollapsed)} title={sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}>
            {#if sidebarCollapsed}
              <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="6,3 11,8 6,13"/>
              </svg>
            {:else}
              <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                <polyline points="10,3 5,8 10,13"/>
              </svg>
            {/if}
          </button>
          {#if !sidebarCollapsed}
            <div class="sidebar-toolbar-actions">
              <button class="icon-btn muted" on:click={openTodayJournal} title="Today's journal">
                <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                  <rect x="1" y="2" width="14" height="13" rx="2"/>
                  <line x1="5" y1="1" x2="5" y2="4"/>
                  <line x1="11" y1="1" x2="11" y2="4"/>
                  <line x1="1" y1="7" x2="15" y2="7"/>
                </svg>
              </button>
              <button class="icon-btn muted" on:click={triggerUpload} title="Upload file">
                <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M13.5 7.5l-6 6a4 4 0 01-5.657-5.657l6.364-6.364a2.5 2.5 0 013.536 3.536L5.379 11.35a1 1 0 01-1.415-1.414l5.657-5.657"/>
                </svg>
              </button>
              <button class="new-btn" on:click={() => (showNewModal = true)} title="New note (⌘N)">
                <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                  <line x1="8" y1="2" x2="8" y2="14"/>
                  <line x1="2" y1="8" x2="14" y2="8"/>
                </svg>
              </button>
              <input bind:this={uploadInput} type="file" multiple style="display:none" on:change={handleUpload} />
            </div>
          {/if}
        </div>

        <div class="tree-scroll">
          <FileTree nodes={$fileTree} {openPaths} on:select={handleSelect} on:delete-folder={handleDeleteFolder} on:delete-file={handleDeleteFile} on:rename={handleRename} />
        </div>

        <div class="sidebar-footer">
          <button class="icon-btn muted sidebar-settings-btn" on:click={openSettings} title="Settings">
            <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor">
              <path d="M9.405 1.05c-.413-1.4-2.397-1.4-2.81 0l-.1.34a1.464 1.464 0 0 1-2.105.872l-.31-.17c-1.283-.698-2.686.705-1.987 1.987l.169.311c.446.82.023 1.841-.872 2.105l-.34.1c-1.4.413-1.4 2.397 0 2.81l.34.1a1.464 1.464 0 0 1 .872 2.105l-.17.31c-.698 1.283.705 2.686 1.987 1.987l.311-.169a1.464 1.464 0 0 1 2.105.872l.1.34c.413 1.4 2.397 1.4 2.81 0l.1-.34a1.464 1.464 0 0 1 2.105-.872l.31.17c1.283.698 2.686-.705 1.987-1.987l-.169-.311a1.464 1.464 0 0 1 .872-2.105l.34-.1c1.4-.413 1.4-2.397 0-2.81l-.34-.1a1.464 1.464 0 0 1-.872-2.105l.17-.31c.698-1.283-.705-2.686-1.987-1.987l-.311.169a1.464 1.464 0 0 1-2.105-.872zM8 10.93a2.929 2.929 0 1 1 0-5.86 2.929 2.929 0 0 1 0 5.858z"/>
            </svg>
          </button>
          <a href="https://github.com/ryantiger658/Hash/releases/tag/v{version}" target="_blank" rel="noopener noreferrer" class="sidebar-meta-link app-version" title="Release notes for v{version}">v{version}</a>
          <a href="https://github.com/ryantiger658/Hash" target="_blank" rel="noopener noreferrer" class="sidebar-meta-link" title="View on GitHub">
            <svg width="13" height="13" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/>
            </svg>
          </a>
          <a href="https://buymeacoffee.com/ryantiger658" target="_blank" rel="noopener noreferrer" class="sidebar-meta-link" title="Buy me a coffee">
            <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 3h8l-1 7H4L3 3z"/>
              <path d="M11 5h1.5a1.5 1.5 0 0 1 0 3H11"/>
              <line x1="5" y1="13" x2="9" y2="13"/>
            </svg>
          </a>
        </div>
      </aside>

      <!-- Floating editor mode panel (right side) — only for markdown files -->
      {#if $selectedPath && !isAsset}
        <div class="editor-mode-float">
          <button class:active={editorMode === 'edit'} on:click={() => (editorMode = 'edit')} title="Edit">
            <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M11.5 2.5l2 2L5 13H3v-2L11.5 2.5z"/>
            </svg>
          </button>
          <button class:active={editorMode === 'split'} on:click={() => (editorMode = 'split')} title="Split">
            <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <rect x="1" y="2" width="6" height="12" rx="1.5"/>
              <rect x="9" y="2" width="6" height="12" rx="1.5"/>
            </svg>
          </button>
          <button class:active={editorMode === 'preview'} on:click={() => (editorMode = 'preview')} title="Preview">
            <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M1 8s3-4.5 7-4.5S15 8 15 8s-3 4.5-7 4.5S1 8 1 8z"/>
              <circle cx="8" cy="8" r="2"/>
            </svg>
          </button>
          <div class="float-divider"></div>
          <div class="float-save-dot"
            class:dirty={$isDirty || $saveStatus === 'saving'}
            class:error={$saveStatus === 'error'}
            title={$isDirty ? 'Unsaved changes' : ($saveStatus === 'idle' ? 'Saved' : saveLabels[$saveStatus])}
          ></div>
        </div>
      {/if}

      <!-- Main content -->
      <main class="main">
        {#if $remoteChangeAvailable}
          <div class="remote-change-banner">
            <span>This file was updated on the server.</span>
            <button class="banner-btn" on:click={acceptRemoteChange}>Reload</button>
            <button class="banner-dismiss" on:click={() => remoteChangeAvailable.set(false)} title="Dismiss">✕</button>
          </div>
        {/if}
        {#if $selectedPath && isAsset}
          <AssetViewer path={$selectedPath} />
        {:else if $selectedPath}
          <Editor file={$selectedFile} bind:mode={editorMode} on:wikilink={handleWikiLink} />
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

  {#if showSettings}
    <SettingsPanel on:close={() => (showSettings = false)} on:poll-interval-change={onPollIntervalChange} />
  {/if}
{/if}

<style>
  /* ── CSS variables (dark default + light override) ───────────────────── */
  :global(:root) {
    --color-bg:         #000000;
    --color-surface:    #0d0d0d;
    --color-border:     #1f1f1f;
    --color-text:       #e2e4ed;
    --color-text-muted: #6b6e85;
    /* --color-accent-raw is set by JS (applyAccentColor); CSS derives --color-accent from it.
       Fallback values here cover the first paint before JS runs. */
    --color-accent-raw: #aaff00;
    --color-accent:     var(--color-accent-raw);
    --color-accent-dim: #aaff0026;
  }

  /* Light mode: darken the accent ~50% so it reads on white/light backgrounds. */
  :global([data-theme="light"]) {
    --color-bg:         #f5f6fa;
    --color-surface:    #ffffff;
    --color-border:     #dde0f0;
    --color-text:       #1a1d2e;
    --color-text-muted: #6b6e85;
    --color-accent:     color-mix(in srgb, var(--color-accent-raw) 50%, #000000);
    --color-accent-dim: color-mix(in srgb, var(--color-accent-raw) 18%, transparent);
  }

  @media (prefers-color-scheme: light) {
    :global(:root:not([data-theme="dark"])) {
      --color-bg:         #f5f6fa;
      --color-surface:    #ffffff;
      --color-border:     #dde0f0;
      --color-text:       #1a1d2e;
      --color-text-muted: #6b6e85;
      --color-accent:     color-mix(in srgb, var(--color-accent-raw) 50%, #000000);
      --color-accent-dim: color-mix(in srgb, var(--color-accent-raw) 18%, transparent);
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
    padding-top: env(safe-area-inset-top, 0px);
    height: calc(48px + env(safe-area-inset-top, 0px));
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
    position: relative;
  }

  .sidebar-expand-float {
    position: absolute;
    top: 8px;
    left: 8px;
    z-index: 5;
    display: none;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: 1px solid var(--color-border);
    border-radius: 5px;
    background: var(--color-surface);
    color: var(--color-text-muted);
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
  }

  .sidebar-expand-float:hover {
    background: var(--color-border);
    color: var(--color-text);
  }

  /* ── Floating editor mode panel (right side) ────────────────────────── */
  .editor-mode-float {
    position: fixed;
    right: 8px;
    top: 56px;
    z-index: 5;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    padding: 4px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.12);
  }

  .editor-mode-float button {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border: none;
    border-radius: 5px;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
  }

  .editor-mode-float button:hover {
    background: var(--color-border);
    color: var(--color-text);
  }

  .editor-mode-float button.active {
    background: var(--color-border);
    color: var(--color-accent);
  }

  .float-divider {
    width: 14px;
    height: 1px;
    background: var(--color-border);
    margin: 2px 0;
  }

  @keyframes dot-pulse {
    0%, 100% { opacity: 1;    transform: scale(1); }
    50%       { opacity: 0.35; transform: scale(0.7); }
  }

  .float-save-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--color-accent); /* default: saved/synced */
    transition: background 0.2s;
    margin: 2px 0;
    flex-shrink: 0;
  }

  .float-save-dot.dirty { background: #f59e0b; animation: dot-pulse 1.4s ease-in-out infinite; }
  .float-save-dot.error { background: #f87171; animation: dot-pulse 1.0s ease-in-out infinite; }

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
    gap: 0.5rem;
  }

  .sidebar-settings-btn {
    padding: 0.15rem 0.25rem;
    margin-right: auto;
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

  /* ── Remote-change conflict banner ───────────────────────────────────── */
  .remote-change-banner {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 1rem;
    background: color-mix(in srgb, var(--color-accent) 12%, var(--color-surface));
    border-bottom: 1px solid color-mix(in srgb, var(--color-accent) 30%, transparent);
    font-size: 0.8rem;
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  .banner-btn {
    padding: 0.15rem 0.6rem;
    font-size: 0.78rem;
    font-family: inherit;
    background: var(--color-accent);
    color: #000;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-weight: 600;
    margin-left: 0.25rem;
  }

  .banner-dismiss {
    margin-left: auto;
    background: none;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    font-size: 0.8rem;
    padding: 0.1rem 0.3rem;
    border-radius: 3px;
  }
  .banner-dismiss:hover { background: var(--color-border); color: var(--color-text); }

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

  .sidebar-toolbar-actions {
    display: flex;
    align-items: center;
    gap: 0.1rem;
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

  /* ── Desktop only ────────────────────────────────────────────────────── */
  @media (min-width: 641px) {
    .collapse-btn { display: inline-flex; }

    /* Collapsed: fully hidden */
    .sidebar.collapsed {
      width: 0;
      min-width: 0;
      overflow: hidden;
      border-right: none;
    }

    .desktop-float { display: inline-flex; }
    .mobile-float  { display: none; }
  }

  /* ── Responsive ──────────────────────────────────────────────────────── */
  @media (max-width: 640px) {
    .desktop-float { display: none; }
    .mobile-float  { display: inline-flex; }

    .sidebar {
      position: fixed;
      top: calc(48px + env(safe-area-inset-top, 0px));
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
      inset: calc(48px + env(safe-area-inset-top, 0px)) 0 0 0;
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
