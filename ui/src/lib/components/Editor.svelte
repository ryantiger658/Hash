<script>
  import { fileContent, saveStatus, isDirty, scheduleAutoSave } from '../../stores/vault.js'
  import { editorLabels } from '../../lib/theme.js'
  import Viewer from './Viewer.svelte'
  import { createEventDispatcher } from 'svelte'

  export let mode = 'split' // 'edit' | 'preview' | 'split'
  export let file = null    // FileEntry — passed to Viewer for metadata footer

  const dispatch = createEventDispatcher()

  function onInput(e) {
    fileContent.set(e.target.value)
    scheduleAutoSave()
  }

  function onWikiLink(e) {
    dispatch('wikilink', e.detail)
  }

  const saveLabels = {
    idle: '',
    saving: 'Saving…',
    saved: 'Saved ✓',
    error: 'Save failed',
  }
</script>

<div class="editor-wrap">
  <div class="toolbar">
    <div class="modes">
      <button class:active={mode === 'edit'}    on:click={() => (mode = 'edit')}    title="Edit">
        <!-- Pencil -->
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
          <path d="M11.5 2.5l2 2L5 13H3v-2L11.5 2.5z"/>
        </svg>
        {#if $editorLabels}<span>Edit</span>{/if}
      </button>
      <button class:active={mode === 'split'}   on:click={() => (mode = 'split')}   title="Split">
        <!-- Two columns -->
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
          <rect x="1" y="2" width="6" height="12" rx="1.5"/>
          <rect x="9" y="2" width="6" height="12" rx="1.5"/>
        </svg>
        {#if $editorLabels}<span>Split</span>{/if}
      </button>
      <button class:active={mode === 'preview'} on:click={() => (mode = 'preview')} title="Preview">
        <!-- Eye -->
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
          <path d="M1 8s3-4.5 7-4.5S15 8 15 8s-3 4.5-7 4.5S1 8 1 8z"/>
          <circle cx="8" cy="8" r="2"/>
        </svg>
        {#if $editorLabels}<span>Preview</span>{/if}
      </button>
    </div>
    <span class="save-status" class:dirty={$isDirty} class:error={$saveStatus === 'error'}>
      {$isDirty && $saveStatus === 'idle' ? '● Unsaved' : saveLabels[$saveStatus]}
    </span>
  </div>

  <div class="panes" class:split={mode === 'split'}>
    {#if mode === 'edit' || mode === 'split'}
      <textarea
        class="raw"
        class:full={mode === 'edit'}
        value={$fileContent}
        on:input={onInput}
        spellcheck="true"
        placeholder="Start writing…"
      ></textarea>
    {/if}

    {#if mode === 'preview' || mode === 'split'}
      <div class="preview" class:full={mode === 'preview'}>
        <Viewer content={$fileContent} {file} on:wikilink={onWikiLink} />
      </div>
    {/if}
  </div>
</div>

<style>
  .editor-wrap {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  /* ── Toolbar ─────────────────────────────────────────────────────────────── */
  .toolbar {
    display: flex;
    align-items: center;
    padding: 0.35rem 0.75rem;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
    flex-shrink: 0;
    gap: 0.5rem;
  }

  .modes {
    margin-left: auto;
    display: flex;
    gap: 2px;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 6px;
    padding: 2px;
  }

  .modes button {
    padding: 0.2rem 0.65rem;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 0.8rem;
    cursor: pointer;
    transition: background 0.1s, color 0.1s;
  }

  .modes button.active {
    background: var(--color-surface);
    color: var(--color-text);
    box-shadow: 0 1px 2px rgba(0,0,0,0.15);
  }

  .save-status {
    font-size: 0.78rem;
    color: var(--color-text-muted);
    min-width: 80px;
    text-align: right;
  }

  .save-status.dirty {
    color: var(--color-accent);
  }

  .save-status.error {
    color: #f87171;
  }

  /* ── Panes ───────────────────────────────────────────────────────────────── */
  .panes {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .panes.split .raw {
    border-right: 1px solid var(--color-border);
  }

  .raw {
    flex: 1;
    padding: 1.25rem 1.5rem;
    border: none;
    background: var(--color-bg);
    color: var(--color-text);
    font-family: ui-monospace, 'Cascadia Code', monospace;
    font-size: 0.9rem;
    line-height: 1.7;
    resize: none;
    outline: none;
    overflow-y: auto;
  }

  .raw.full {
    max-width: 780px;
    margin: 0 auto;
  }

  .preview {
    flex: 1;
    overflow-y: auto;
    background: var(--color-bg);
  }

  .preview.full {
    width: 100%;
  }
</style>
