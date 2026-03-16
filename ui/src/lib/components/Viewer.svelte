<script>
  import { createEventDispatcher } from 'svelte'
  import { renderMarkdown } from '../markdown.js'

  export let content = ''
  /** FileEntry from the server — provides created/modified timestamps. */
  export let file = null

  const dispatch = createEventDispatcher()

  $: html = renderMarkdown(content)

  function fmtDate(unix) {
    if (!unix) return null
    return new Date(unix * 1000).toLocaleString(undefined, {
      year: 'numeric', month: 'short', day: 'numeric',
      hour: '2-digit', minute: '2-digit'
    })
  }

  $: createdStr  = file ? fmtDate(file.created)  : null
  $: modifiedStr = file ? fmtDate(file.modified) : null

  // Handle clicks on wiki-links inside the rendered HTML
  function handleClick(e) {
    const link = e.target.closest('.wiki-link')
    if (link) {
      e.preventDefault()
      const target = decodeURIComponent(link.dataset.target ?? '')
      if (target) dispatch('wikilink', target)
    }
  }

  // Handle checkbox toggles — dispatch event so the editor can patch the source
  function handleChange(e) {
    const cb = e.target.closest('input[type="checkbox"][data-cb]')
    if (cb) {
      const index = parseInt(cb.dataset.cb, 10)
      dispatch('checkbox-toggle', { index, checked: cb.checked })
    }
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="viewer prose" on:click={handleClick} on:change={handleChange}>
  {@html html}

  {#if modifiedStr}
    <footer class="meta-footer">
      {#if createdStr}<span>Created {createdStr}</span>{/if}
      {#if createdStr && modifiedStr}<span class="sep">·</span>{/if}
      {#if modifiedStr}<span>Updated {modifiedStr}</span>{/if}
    </footer>
  {/if}
</div>

<style>
  .viewer {
    padding: 1.5rem 2rem;
    max-width: 780px;
    margin: 0 auto;
    outline: none;
  }

  /* ── Prose styles ────────────────────────────────────────────────────────── */
  .viewer :global(h1),
  .viewer :global(h2),
  .viewer :global(h3),
  .viewer :global(h4) {
    color: var(--color-text);
    margin: 1.5em 0 0.5em;
    line-height: 1.3;
  }

  .viewer :global(h1) { font-size: 1.8rem; border-bottom: 1px solid var(--color-border); padding-bottom: 0.3em; }
  .viewer :global(h2) { font-size: 1.4rem; border-bottom: 1px solid var(--color-border); padding-bottom: 0.2em; }
  .viewer :global(h3) { font-size: 1.15rem; }

  .viewer :global(p) {
    margin: 0.75em 0;
    line-height: 1.7;
    color: var(--color-text);
  }

  .viewer :global(a) {
    color: var(--color-accent);
    text-decoration: none;
  }

  .viewer :global(a:hover) {
    text-decoration: underline;
  }

  .viewer :global(.wiki-link) {
    color: var(--color-accent);
    border-bottom: 1px dashed var(--color-accent);
    cursor: pointer;
  }

  .viewer :global(code) {
    background: var(--color-border);
    padding: 0.15em 0.4em;
    border-radius: 4px;
    font-size: 0.88em;
    font-family: ui-monospace, monospace;
  }

  .viewer :global(pre) {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    padding: 1rem 1.25rem;
    overflow-x: auto;
    margin: 1rem 0;
  }

  .viewer :global(pre code) {
    background: none;
    padding: 0;
    font-size: 0.875rem;
  }

  .viewer :global(blockquote) {
    border-left: 3px solid var(--color-accent);
    margin: 1rem 0;
    padding: 0.5rem 1rem;
    color: var(--color-text-muted);
    background: var(--color-surface);
    border-radius: 0 6px 6px 0;
  }

  .viewer :global(ul),
  .viewer :global(ol) {
    margin: 0.75em 0;
    padding-left: 1.5rem;
  }

  .viewer :global(li) {
    margin: 0.3em 0;
    line-height: 1.6;
  }

  /* Task list checkboxes */
  .viewer :global(input[type="checkbox"]) {
    margin-right: 0.4em;
    accent-color: var(--color-accent);
  }

  .viewer :global(table) {
    border-collapse: collapse;
    width: 100%;
    margin: 1rem 0;
    font-size: 0.9rem;
  }

  .viewer :global(th),
  .viewer :global(td) {
    border: 1px solid var(--color-border);
    padding: 0.5rem 0.75rem;
    text-align: left;
  }

  .viewer :global(th) {
    background: var(--color-surface);
    font-weight: 600;
  }

  .viewer :global(tr:nth-child(even)) {
    background: var(--color-surface);
  }

  .viewer :global(hr) {
    border: none;
    border-top: 1px solid var(--color-border);
    margin: 1.5rem 0;
  }

  .viewer :global(img) {
    max-width: 100%;
    border-radius: 6px;
  }

  .meta-footer {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    margin-top: 3rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--color-border);
    font-size: 0.75rem;
    color: var(--color-text-muted);
    user-select: none;
  }

  .sep {
    opacity: 0.4;
  }

  /* ── Color chip (hex color preview in inline code) ────────────────────── */
  .viewer :global(.color-chip) {
    display: inline-block;
    width: 0.75em;
    height: 0.75em;
    border-radius: 2px;
    margin-right: 0.3em;
    vertical-align: middle;
    border: 1px solid rgba(128,128,128,0.3);
    flex-shrink: 0;
  }

  /* ── Syntax highlighting (hljs — adapts to light/dark via CSS vars) ───── */
  .viewer :global(.hljs)                 { color: var(--color-text); }
  .viewer :global(.hljs-comment),
  .viewer :global(.hljs-quote)           { color: var(--color-text-muted); font-style: italic; }
  .viewer :global(.hljs-keyword),
  .viewer :global(.hljs-selector-tag),
  .viewer :global(.hljs-subst)           { color: #c792ea; }
  .viewer :global(.hljs-string),
  .viewer :global(.hljs-doctag),
  .viewer :global(.hljs-regexp)          { color: #c3e88d; }
  .viewer :global(.hljs-number),
  .viewer :global(.hljs-literal),
  .viewer :global(.hljs-variable),
  .viewer :global(.hljs-template-variable),
  .viewer :global(.hljs-tag .hljs-attr) { color: #f78c6c; }
  .viewer :global(.hljs-title),
  .viewer :global(.hljs-section),
  .viewer :global(.hljs-selector-id),
  .viewer :global(.hljs-title.class_)   { color: #82aaff; }
  .viewer :global(.hljs-type),
  .viewer :global(.hljs-class .hljs-title),
  .viewer :global(.hljs-built_in)       { color: #ffcb6b; }
  .viewer :global(.hljs-attr),
  .viewer :global(.hljs-attribute)      { color: #89ddff; }
  .viewer :global(.hljs-meta),
  .viewer :global(.hljs-meta .hljs-keyword) { color: #f07178; }
  .viewer :global(.hljs-addition)       { color: #c3e88d; background: rgba(195,232,141,0.1); }
  .viewer :global(.hljs-deletion)       { color: #f07178; background: rgba(240,113,120,0.1); }
  .viewer :global(.hljs-emphasis)       { font-style: italic; }
  .viewer :global(.hljs-strong)         { font-weight: bold; }
</style>
