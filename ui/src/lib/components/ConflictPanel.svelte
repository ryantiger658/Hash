<script>
  import { createEventDispatcher, onMount } from 'svelte'

  const dispatch = createEventDispatcher()

  let conflicts = []
  let resolving = {}
  let error = ''

  function tauriInvoke(cmd, args) {
    return window.__TAURI__?.core?.invoke(cmd, args)
  }

  async function loadConflicts() {
    try {
      conflicts = (await tauriInvoke('get_conflicts')) ?? []
    } catch (e) {
      error = String(e)
    }
  }

  async function resolve(path, resolution) {
    resolving = { ...resolving, [path]: true }
    error = ''
    try {
      await tauriInvoke('resolve_conflict', { path, resolution })
      conflicts = conflicts.filter(c => c.path !== path)
      dispatch('resolved')
      if (conflicts.length === 0) dispatch('close')
    } catch (e) {
      error = String(e)
    } finally {
      resolving = { ...resolving, [path]: false }
    }
  }

  function formatAge(ts) {
    const diff = Math.floor(Date.now() / 1000 - ts)
    if (diff < 60) return 'just now'
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`
    return new Date(ts * 1000).toLocaleDateString()
  }

  onMount(loadConflicts)
</script>

<div class="overlay" role="dialog" aria-modal="true">
  <div class="panel">
    <div class="panel-header">
      <div class="panel-title">
        <!-- Warning icon -->
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" class="title-icon">
          <path d="M8 2L14.5 13H1.5L8 2z"/>
          <line x1="8" y1="7" x2="8" y2="10"/>
          <circle cx="8" cy="12" r="0.5" fill="currentColor" stroke="none"/>
        </svg>
        {conflicts.length} Sync {conflicts.length === 1 ? 'Conflict' : 'Conflicts'}
      </div>
      <button class="close-btn" on:click={() => dispatch('close')} title="Close">
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
          <line x1="2" y1="2" x2="14" y2="14"/>
          <line x1="14" y1="2" x2="2" y2="14"/>
        </svg>
      </button>
    </div>

    <p class="panel-desc">
      Both you and the server changed these files since your last sync.
      Choose which version to keep — the other will be discarded.
    </p>

    {#if error}
      <p class="error-msg">{error}</p>
    {/if}

    <div class="conflicts-list">
      {#each conflicts as conflict (conflict.path)}
        <div class="conflict-item">
          <div class="conflict-meta">
            <span class="conflict-path">{conflict.path}</span>
            <span class="conflict-age">detected {formatAge(conflict.detected_at)}</span>
          </div>

          <div class="diff-panes">
            <div class="diff-pane local">
              <div class="pane-label">
                <svg width="10" height="10" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                  <circle cx="8" cy="8" r="6"/>
                  <line x1="8" y1="5" x2="8" y2="8"/>
                  <line x1="8" y1="8" x2="11" y2="8"/>
                </svg>
                Your changes
              </div>
              <pre class="pane-content">{conflict.local_content || '(empty)'}</pre>
            </div>
            <div class="diff-pane server">
              <div class="pane-label">
                <!-- Server icon -->
                <svg width="10" height="10" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
                  <rect x="1" y="2" width="14" height="4" rx="1.5"/>
                  <rect x="1" y="9" width="14" height="4" rx="1.5"/>
                </svg>
                Server version
              </div>
              <pre class="pane-content">{conflict.server_content || '(empty)'}</pre>
            </div>
          </div>

          <div class="conflict-actions">
            <button
              class="btn-keep btn-keep-mine"
              disabled={resolving[conflict.path]}
              on:click={() => resolve(conflict.path, 'local')}
            >
              {resolving[conflict.path] ? 'Resolving…' : 'Keep Mine'}
            </button>
            <button
              class="btn-keep btn-keep-server"
              disabled={resolving[conflict.path]}
              on:click={() => resolve(conflict.path, 'server')}
            >
              {resolving[conflict.path] ? 'Resolving…' : 'Keep Server'}
            </button>
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>

<style>
  .overlay {
    position: fixed;
    inset: 0;
    z-index: 400;
    background: rgba(0, 0, 0, 0.55);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: 48px 1rem 1rem;
  }

  .panel {
    width: 100%;
    max-width: 800px;
    max-height: calc(100vh - 64px);
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 10px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.9rem 1rem 0.7rem;
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .panel-title {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .title-icon {
    color: #f59e0b;
    flex-shrink: 0;
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: 4px;
    transition: background 0.1s, color 0.1s;
  }

  .close-btn:hover { background: var(--color-border); color: var(--color-text); }

  .panel-desc {
    padding: 0.6rem 1rem;
    font-size: 0.78rem;
    color: var(--color-text-muted);
    flex-shrink: 0;
    border-bottom: 1px solid var(--color-border);
  }

  .error-msg {
    margin: 0.5rem 1rem;
    padding: 0.4rem 0.6rem;
    font-size: 0.78rem;
    color: #f87171;
    background: color-mix(in srgb, #f87171 10%, transparent);
    border-radius: 4px;
    border: 1px solid color-mix(in srgb, #f87171 25%, transparent);
    flex-shrink: 0;
  }

  .conflicts-list {
    overflow-y: auto;
    flex: 1;
    padding: 0.75rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .conflict-item {
    border: 1px solid var(--color-border);
    border-radius: 7px;
    overflow: hidden;
  }

  .conflict-meta {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    background: var(--color-bg);
    border-bottom: 1px solid var(--color-border);
  }

  .conflict-path {
    font-family: 'Hack', monospace;
    font-size: 0.78rem;
    color: var(--color-text);
    font-weight: 600;
  }

  .conflict-age {
    font-size: 0.72rem;
    color: var(--color-text-muted);
  }

  .diff-panes {
    display: grid;
    grid-template-columns: 1fr 1fr;
  }

  .diff-pane {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  .diff-pane.local { border-right: 1px solid var(--color-border); }

  .pane-label {
    display: flex;
    align-items: center;
    gap: 0.3rem;
    padding: 0.3rem 0.6rem;
    font-size: 0.7rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    border-bottom: 1px solid var(--color-border);
  }

  .diff-pane.local .pane-label { color: #f59e0b; background: color-mix(in srgb, #f59e0b 8%, var(--color-surface)); }
  .diff-pane.server .pane-label { color: var(--color-accent); background: color-mix(in srgb, var(--color-accent) 8%, var(--color-surface)); }

  .pane-content {
    font-family: 'Hack', ui-monospace, monospace;
    font-size: 0.72rem;
    line-height: 1.5;
    padding: 0.5rem 0.6rem;
    margin: 0;
    white-space: pre;
    overflow-x: auto;
    max-height: 220px;
    overflow-y: auto;
    color: var(--color-text);
    background: var(--color-surface);
    flex: 1;
  }

  .conflict-actions {
    display: flex;
    gap: 0.5rem;
    padding: 0.6rem 0.75rem;
    border-top: 1px solid var(--color-border);
    background: var(--color-bg);
    justify-content: flex-end;
  }

  .btn-keep {
    padding: 0.3rem 0.9rem;
    font-size: 0.8rem;
    font-weight: 600;
    font-family: inherit;
    border-radius: 5px;
    border: none;
    cursor: pointer;
    transition: filter 0.1s, opacity 0.1s;
  }

  .btn-keep:disabled { opacity: 0.5; cursor: default; }
  .btn-keep:not(:disabled):hover { filter: brightness(1.1); }

  .btn-keep-mine {
    background: color-mix(in srgb, #f59e0b 15%, var(--color-surface));
    color: #f59e0b;
    border: 1px solid color-mix(in srgb, #f59e0b 35%, transparent);
  }

  .btn-keep-server {
    background: var(--color-accent);
    color: #000;
  }

  /* Narrow: stack panes vertically */
  @media (max-width: 560px) {
    .diff-panes { grid-template-columns: 1fr; }
    .diff-pane.local { border-right: none; border-bottom: 1px solid var(--color-border); }
  }
</style>
