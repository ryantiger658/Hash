<script>
  import { createEventDispatcher } from 'svelte'
  import { renderMarkdown } from '../markdown.js'

  export let content = ''

  const dispatch = createEventDispatcher()

  $: html = renderMarkdown(content)

  // Handle clicks on wiki-links inside the rendered HTML
  function handleClick(e) {
    const link = e.target.closest('.wiki-link')
    if (link) {
      e.preventDefault()
      const target = decodeURIComponent(link.dataset.target ?? '')
      if (target) dispatch('wikilink', target)
    }
  }
</script>

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="viewer prose" on:click={handleClick}>
  {@html html}
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
</style>
