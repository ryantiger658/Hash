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

  // Track which folders are open
  let open = {}

  function toggle(node) {
    open[node.path] = !open[node.path]
  }

  function select(node) {
    dispatch('select', node.path)
  }

  // Bubble events from nested trees up to the root
  function bubble(e) {
    dispatch('select', e.detail)
  }
</script>

<ul class="tree" class:root={depth === 0}>
  {#each nodes as node (node.path)}
    <li>
      {#if node.isDir}
        <button
          class="tree-row folder"
          style="padding-left: {depth * 14 + 8}px"
          on:click={() => toggle(node)}
          aria-expanded={!!open[node.path]}
        >
          <span class="arrow" class:open={open[node.path]}>›</span>
          <span class="icon">📁</span>
          <span class="name">{node.name}</span>
        </button>
        {#if open[node.path]}
          <svelte:self nodes={node.children} depth={depth + 1} on:select={bubble} />
        {/if}
      {:else}
        <button
          class="tree-row file"
          class:active={$selectedPath === node.path}
          style="padding-left: {depth * 14 + 8}px"
          on:click={() => select(node)}
        >
          <span class="icon">📄</span>
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
    background: var(--color-accent-dim);
    color: var(--color-accent);
    font-weight: 500;
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
