<script>
  import { fileContent, saveStatus, isDirty, scheduleAutoSave } from '../../stores/vault.js'
  import Viewer from './Viewer.svelte'
  import { createEventDispatcher } from 'svelte'

  export let mode = 'split' // 'edit' | 'preview' | 'split'

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
      <button class:active={mode === 'edit'}    on:click={() => (mode = 'edit')}>Edit</button>
      <button class:active={mode === 'split'}   on:click={() => (mode = 'split')}>Split</button>
      <button class:active={mode === 'preview'} on:click={() => (mode = 'preview')}>Preview</button>
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
        <Viewer content={$fileContent} on:wikilink={onWikiLink} />
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
    justify-content: space-between;
    padding: 0.35rem 0.75rem;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
    flex-shrink: 0;
  }

  .modes {
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
