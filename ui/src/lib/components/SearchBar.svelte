<script>
  import { createEventDispatcher } from 'svelte'
  import { api } from '../api.js'

  const dispatch = createEventDispatcher()

  let query = ''
  let results = []
  let searching = false
  let timer = null

  function onInput() {
    clearTimeout(timer)
    if (!query.trim()) {
      results = []
      return
    }
    timer = setTimeout(runSearch, 300)
  }

  async function runSearch() {
    searching = true
    try {
      results = await api.search(query)
    } finally {
      searching = false
    }
  }

  function selectResult(path) {
    query = ''
    results = []
    dispatch('select', path)
  }

  function clear() {
    query = ''
    results = []
  }
</script>

<div class="search-wrap">
  <div class="input-row">
    <span class="icon">🔍</span>
    <input
      type="search"
      bind:value={query}
      on:input={onInput}
      placeholder="Search notes…"
    />
    {#if query}
      <button class="clear" on:click={clear} aria-label="Clear">✕</button>
    {/if}
  </div>

  {#if results.length > 0}
    <ul class="results">
      {#each results as r}
        <li>
          <button on:click={() => selectResult(r.path)}>
            <span class="result-path">{r.path.replace(/\.md$/, '')}</span>
            {#if r.snippet}
              <span class="result-snippet">{r.snippet}</span>
            {/if}
          </button>
        </li>
      {/each}
    </ul>
  {:else if query && !searching}
    <p class="no-results">No results for "{query}"</p>
  {/if}
</div>

<style>
  .search-wrap {
    position: relative;
    flex: 1;
    max-width: 400px;
  }

  .input-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    padding: 0.25rem 0.6rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg);
    transition: border-color 0.15s;
  }

  .input-row:focus-within {
    border-color: var(--color-accent);
  }

  .icon {
    font-size: 0.85rem;
    flex-shrink: 0;
  }

  input {
    border: none;
    background: transparent;
    color: var(--color-text);
    font-size: 0.85rem;
    width: 100%;
    outline: none;
  }

  input::placeholder {
    color: var(--color-text-muted);
  }

  .clear {
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 0.75rem;
    cursor: pointer;
    padding: 0;
    line-height: 1;
  }

  .results {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    list-style: none;
    margin: 0;
    padding: 0.25rem;
    z-index: 100;
    max-height: 320px;
    overflow-y: auto;
    box-shadow: 0 8px 24px rgba(0,0,0,0.2);
  }

  .results li button {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    padding: 0.45rem 0.6rem;
    border: none;
    border-radius: 5px;
    background: transparent;
    text-align: left;
    cursor: pointer;
    transition: background 0.1s;
  }

  .results li button:hover {
    background: var(--color-border);
  }

  .result-path {
    font-size: 0.85rem;
    color: var(--color-text);
    font-weight: 500;
  }

  .result-snippet {
    font-size: 0.78rem;
    color: var(--color-text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  .no-results {
    position: absolute;
    top: calc(100% + 4px);
    left: 0;
    right: 0;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 8px;
    padding: 0.75rem;
    font-size: 0.85rem;
    color: var(--color-text-muted);
    text-align: center;
    z-index: 100;
  }
</style>
