<script>
  import { onMount } from 'svelte'
  import { loadServerTheme, setTheme, getStoredTheme, THEMES } from './lib/theme.js'

  let currentTheme = getStoredTheme()

  onMount(() => {
    loadServerTheme()
  })

  function cycleTheme() {
    const order = ['light', 'dark', 'system']
    const next = order[(order.indexOf(currentTheme) + 1) % order.length]
    currentTheme = next
    setTheme(next)
  }

  const themeIcon = { light: '☀️', dark: '🌙', system: '💻' }
  const themeLabel = { light: 'Light', dark: 'Dark', system: 'System' }
</script>

<main>
  <header>
    <span class="logo">#ash</span>
    <nav class="header-actions">
      <button
        class="theme-toggle"
        on:click={cycleTheme}
        title="Theme: {themeLabel[currentTheme]}"
        aria-label="Toggle theme"
      >
        {themeIcon[currentTheme]}
        <span class="theme-label">{themeLabel[currentTheme]}</span>
      </button>
    </nav>
  </header>

  <div class="layout">
    <aside class="sidebar">
      <!-- M1: file tree -->
      <p class="placeholder">File tree coming in M1</p>
    </aside>

    <section class="content">
      <!-- M1: markdown viewer / editor -->
      <p class="placeholder">Select a file to begin.</p>
    </section>
  </div>
</main>

<style>
  /* ── CSS custom properties ───────────────────────────────────────────────── */
  /* Dark theme (default) */
  :global(:root) {
    --color-bg:        #0f1117;
    --color-surface:   #1a1d27;
    --color-border:    #2a2d3d;
    --color-text:      #e2e4ed;
    --color-text-muted:#8b8fa8;
    --color-accent:    #6366f1;      /* overridden by server config */
    --color-accent-dim:#6366f1aa;
  }

  /* Light theme — activated by data-theme="light" on <html> */
  :global([data-theme="light"]) {
    --color-bg:        #f5f6fa;
    --color-surface:   #ffffff;
    --color-border:    #dde0f0;
    --color-text:      #1a1d2e;
    --color-text-muted:#6b6e85;
  }

  /* System preference fallback when no data-theme attribute is set */
  @media (prefers-color-scheme: light) {
    :global(:root:not([data-theme="dark"])) {
      --color-bg:        #f5f6fa;
      --color-surface:   #ffffff;
      --color-border:    #dde0f0;
      --color-text:      #1a1d2e;
      --color-text-muted:#6b6e85;
    }
  }

  /* ── Global resets ───────────────────────────────────────────────────────── */
  :global(*, *::before, *::after) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(body) {
    font-family: system-ui, -apple-system, sans-serif;
    background: var(--color-bg);
    color: var(--color-text);
    height: 100vh;
    transition: background 0.2s ease, color 0.2s ease;
  }

  /* ── Layout ──────────────────────────────────────────────────────────────── */
  main {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 1.25rem;
    height: 48px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .logo {
    font-weight: 700;
    font-size: 1.1rem;
    color: var(--color-accent);
    letter-spacing: 0.02em;
  }

  .header-actions {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }

  .layout {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .sidebar {
    width: 240px;
    min-width: 180px;
    background: var(--color-surface);
    border-right: 1px solid var(--color-border);
    padding: 1rem;
    overflow-y: auto;
  }

  .content {
    flex: 1;
    padding: 1.5rem;
    overflow-y: auto;
  }

  /* ── Components ──────────────────────────────────────────────────────────── */
  .theme-toggle {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    padding: 0.3rem 0.65rem;
    border-radius: 6px;
    border: 1px solid var(--color-border);
    background: transparent;
    color: var(--color-text-muted);
    font-size: 0.8rem;
    cursor: pointer;
    transition: border-color 0.15s, color 0.15s;
  }

  .theme-toggle:hover {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .placeholder {
    color: var(--color-text-muted);
    font-size: 0.875rem;
  }
</style>
