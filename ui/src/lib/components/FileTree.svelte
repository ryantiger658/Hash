<script>
  /**
   * Recursive file tree — renders a single level of nodes.
   * Uses svelte:self for nested folders.
   */
  import { createEventDispatcher } from 'svelte'
  import { selectedPath } from '../../stores/vault.js'

  export let nodes = []
  export let depth = 0

  const dispatch = createEventDispatcher()

  let open = {}

  function toggle(node) { open[node.path] = !open[node.path] }
  function select(node) { dispatch('select', node.path) }
  function bubble(e)    { dispatch('select', e.detail) }
  function bubbleDelete(e) { dispatch('delete-folder', e.detail) }

  function confirmDeleteFolder(node) {
    const msg = `Delete folder "${node.name}" and all its contents? This cannot be undone.`
    if (confirm(msg)) dispatch('delete-folder', node.path)
  }
</script>

<ul class="tree" class:root={depth === 0}>
  {#each nodes as node (node.path)}
    <li>
      {#if node.isDir}
        <div class="folder-row-wrap">
          <button
            class="tree-row folder"
            style="padding-left: {depth * 14 + 8}px"
            on:click={() => toggle(node)}
            aria-expanded={!!open[node.path]}
            title={node.name}
          >
            <span class="arrow" class:open={open[node.path]}>›</span>
            <span class="icon">
              <!-- Folder -->
              <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M2 5a1 1 0 011-1h3l1.5 1.5H13a1 1 0 011 1V12a1 1 0 01-1 1H3a1 1 0 01-1-1V5z"/>
              </svg>
            </span>
            <span class="name">{node.name}</span>
          </button>
          <button
            class="folder-delete"
            title="Delete folder"
            on:click|stopPropagation={() => confirmDeleteFolder(node)}
          >
            <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M2 4h12"/>
              <path d="M5 4V2h6v2"/>
              <path d="M3 4l1 10h8l1-10"/>
            </svg>
          </button>
        </div>
        {#if open[node.path]}
          <svelte:self nodes={node.children} depth={depth + 1} on:select={bubble} on:delete-folder={bubbleDelete} />
        {/if}
      {:else}
        <button
          class="tree-row file"
          class:active={$selectedPath === node.path}
          style="padding-left: {depth * 14 + 8}px"
          on:click={() => select(node)}
          title={node.name}
        >
          <span class="icon">
            <!-- Document -->
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
              <path d="M4 2h6l3 3v9a1 1 0 01-1 1H4a1 1 0 01-1-1V3a1 1 0 011-1z"/>
              <path d="M10 2v3h3"/>
            </svg>
          </span>
          <span class="name">{node.name.replace(/\.md$/, '')}</span>
        </button>
      {/if}
    </li>
  {/each}
</ul>

<style>
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  ul.root {
    padding: 0.25rem 0;
  }

  li {
    display: flex;
    flex-direction: column;
  }

  .folder-row-wrap {
    display: flex;
    align-items: center;
    position: relative;
  }

  .folder-row-wrap .tree-row {
    flex: 1;
    min-width: 0;
  }

  .folder-delete {
    display: none;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: 22px;
    height: 22px;
    margin-right: 4px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    padding: 0;
    transition: color 0.1s, background 0.1s;
  }

  .folder-row-wrap:hover .folder-delete {
    display: flex;
  }

  .folder-delete:hover {
    color: #f87171;
    background: var(--color-border);
  }

  .tree-row {
    display: flex;
    align-items: center;
    gap: 5px;
    width: 100%;
    padding-top: 3px;
    padding-bottom: 3px;
    padding-right: 8px;
    border: none;
    border-radius: 5px;
    background: transparent;
    color: var(--color-text);
    font-size: 0.85rem;
    text-align: left;
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    transition: background 0.1s;
  }

  .tree-row:hover {
    background: var(--color-border);
  }

  .tree-row.active {
    background: transparent;
    color: var(--color-accent);
    font-weight: 500;
    box-shadow: inset 0 0 0 1.5px var(--color-accent);
  }

  .arrow {
    display: inline-block;
    width: 12px;
    font-size: 0.75rem;
    transition: transform 0.15s;
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  .arrow.open {
    transform: rotate(90deg);
  }

  .icon {
    font-size: 0.85rem;
    flex-shrink: 0;
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
