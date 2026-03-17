<script>
  /**
   * Recursive file tree — renders a single level of nodes.
   * Uses svelte:self for nested folders.
   */
  import { createEventDispatcher } from 'svelte'
  import { selectedPath } from '../../stores/vault.js'

  export let nodes = []
  export let depth = 0
  export let openPaths = new Set()

  const dispatch = createEventDispatcher()

  let open = {}
  let renamingPath = null   // path of the node currently being renamed
  let renameValue = ''      // current value of the rename input

  // Auto-expand any folder whose path is an ancestor of the selected file
  $: {
    const next = { ...open }
    let changed = false
    for (const node of nodes) {
      if (node.isDir && openPaths.has(node.path) && !next[node.path]) {
        next[node.path] = true
        changed = true
      }
    }
    if (changed) open = next
  }

  function toggle(node) { open[node.path] = !open[node.path] }
  function select(node) { dispatch('select', node.path) }
  function bubble(e)           { dispatch('select', e.detail) }
  function bubbleDelete(e)     { dispatch('delete-folder', e.detail) }
  function bubbleDeleteFile(e) { dispatch('delete-file',   e.detail) }
  function bubbleRename(e)     { dispatch('rename', e.detail) }

  function confirmDeleteFolder(node) {
    const msg = `Delete folder "${node.name}" and all its contents? This cannot be undone.`
    if (confirm(msg)) dispatch('delete-folder', node.path)
  }

  function confirmDeleteFile(node) {
    const msg = `Delete "${node.name}"? This cannot be undone.`
    if (confirm(msg)) dispatch('delete-file', node.path)
  }

  function startRename(node) {
    renamingPath = node.path
    // Pre-fill without .md extension for files
    renameValue = node.isDir ? node.name : node.name.replace(/\.md$/, '')
  }

  function commitRename(node) {
    const newName = renameValue.trim()
    renamingPath = null
    if (!newName || newName === (node.isDir ? node.name : node.name.replace(/\.md$/, ''))) return
    // Build the new full path: same parent directory, new name
    const dir = node.path.includes('/') ? node.path.slice(0, node.path.lastIndexOf('/') + 1) : ''
    const newPath = node.isDir ? dir + newName : dir + newName + (newName.endsWith('.md') ? '' : '.md')
    dispatch('rename', { from: node.path, to: newPath })
  }

  function onRenameKeydown(e, node) {
    if (e.key === 'Enter') { e.preventDefault(); commitRename(node) }
    if (e.key === 'Escape') { renamingPath = null }
  }

  function focusOnMount(el) { el.focus(); el.select() }
</script>

<ul class="tree" class:root={depth === 0}>
  {#each nodes as node (node.path)}
    <li>
      {#if node.isDir}
        <div class="folder-row-wrap">
          {#if renamingPath === node.path}
            <span class="icon" style="padding-left: {depth * 14 + 8}px; flex-shrink:0; display:flex; align-items:center; gap:5px;">
              <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M2 5a1 1 0 011-1h3l1.5 1.5H13a1 1 0 011 1V12a1 1 0 01-1 1H3a1 1 0 01-1-1V5z"/>
              </svg>
            </span>
            <input
              class="rename-input"
              bind:value={renameValue}
              on:keydown={(e) => onRenameKeydown(e, node)}
              on:blur={() => commitRename(node)}
              use:focusOnMount
            />
          {:else}
            <button
              class="tree-row folder"
              style="padding-left: {depth * 14 + 8}px"
              on:click={() => toggle(node)}
              aria-expanded={!!open[node.path]}
              title={node.name}
            >
              <span class="arrow" class:open={open[node.path]}>›</span>
              <span class="icon">
                <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M2 5a1 1 0 011-1h3l1.5 1.5H13a1 1 0 011 1V12a1 1 0 01-1 1H3a1 1 0 01-1-1V5z"/>
                </svg>
              </span>
              <span class="name">{node.name}</span>
            </button>
            <button class="row-action rename-btn" title="Rename" on:click|stopPropagation={() => startRename(node)}>
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M11.5 2.5a1.414 1.414 0 012 2L5 13H3v-2L11.5 2.5z"/>
              </svg>
            </button>
            <button class="row-action folder-delete" title="Delete folder" on:click|stopPropagation={() => confirmDeleteFolder(node)}>
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M2 4h12"/>
                <path d="M5 4V2h6v2"/>
                <path d="M3 4l1 10h8l1-10"/>
              </svg>
            </button>
          {/if}
        </div>
        {#if open[node.path]}
          <svelte:self nodes={node.children} depth={depth + 1} {openPaths} on:select={bubble} on:delete-folder={bubbleDelete} on:delete-file={bubbleDeleteFile} on:rename={bubbleRename} />
        {/if}
      {:else}
        <div class="file-row-wrap" class:active={$selectedPath === node.path}>
          {#if renamingPath === node.path}
            <span class="icon" style="padding-left: {depth * 14 + 8}px; flex-shrink:0; display:flex; align-items:center; gap:5px; color: var(--color-text-muted);">
              <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M4 2h6l3 3v9a1 1 0 01-1 1H4a1 1 0 01-1-1V3a1 1 0 011-1z"/>
                <path d="M10 2v3h3"/>
              </svg>
            </span>
            <input
              class="rename-input"
              bind:value={renameValue}
              on:keydown={(e) => onRenameKeydown(e, node)}
              on:blur={() => commitRename(node)}
              use:focusOnMount
            />
          {:else}
            <button
              class="tree-row file"
              class:active={$selectedPath === node.path}
              style="padding-left: {depth * 14 + 8}px"
              on:click={() => select(node)}
              title={node.name}
            >
              <span class="icon">
                <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                  <path d="M4 2h6l3 3v9a1 1 0 01-1 1H4a1 1 0 01-1-1V3a1 1 0 011-1z"/>
                  <path d="M10 2v3h3"/>
                </svg>
              </span>
              <span class="name">{node.name.replace(/\.md$/, '')}</span>
            </button>
            <button class="row-action rename-btn" title="Rename" on:click|stopPropagation={() => startRename(node)}>
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M11.5 2.5a1.414 1.414 0 012 2L5 13H3v-2L11.5 2.5z"/>
              </svg>
            </button>
            <button class="row-action file-delete" title="Delete file" on:click|stopPropagation={() => confirmDeleteFile(node)}>
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
                <path d="M2 4h12"/>
                <path d="M5 4V2h6v2"/>
                <path d="M3 4l1 10h8l1-10"/>
              </svg>
            </button>
          {/if}
        </div>
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

  .folder-row-wrap,
  .file-row-wrap {
    display: flex;
    align-items: center;
    position: relative;
  }

  .folder-row-wrap .tree-row,
  .file-row-wrap .tree-row {
    flex: 1;
    min-width: 0;
  }

  .row-action {
    display: none;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    width: 22px;
    height: 22px;
    margin-right: 2px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    padding: 0;
    transition: color 0.1s, background 0.1s;
  }

  .folder-row-wrap:hover .row-action,
  .file-row-wrap:hover .row-action {
    display: flex;
  }

  .rename-btn:hover {
    color: var(--color-accent);
    background: var(--color-border);
  }

  .folder-delete:hover,
  .file-delete:hover {
    color: #f87171;
    background: var(--color-border);
  }

  .rename-input {
    flex: 1;
    min-width: 0;
    margin: 1px 4px 1px 2px;
    padding: 2px 6px;
    font-size: 0.85rem;
    font-family: inherit;
    background: var(--color-bg);
    color: var(--color-text);
    border: 1.5px solid var(--color-accent);
    border-radius: 4px;
    outline: none;
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
  }

  .file-row-wrap.active {
    box-shadow: inset 0 0 0 1.5px var(--color-accent);
    border-radius: 5px;
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
