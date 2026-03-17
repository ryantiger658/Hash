<script>
  import { fileContent, scheduleAutoSave } from '../../stores/vault.js'
  import { lineNumbers, spellCheck } from '../theme.js'
  import Viewer from './Viewer.svelte'
  import { createEventDispatcher } from 'svelte'

  export let mode = 'split' // 'edit' | 'preview' | 'split'
  export let file = null    // FileEntry — passed to Viewer for metadata footer

  const dispatch = createEventDispatcher()

  // ── Scroll sync (split mode + gutter) ────────────────────────────────────
  let rawEl
  let previewEl
  let gutterEl
  let syncing = false

  $: lineCount = ($fileContent.match(/\n/g) ?? []).length + 1

  function onRawScroll() {
    if (!rawEl || syncing) return
    syncing = true
    if (gutterEl) gutterEl.scrollTop = rawEl.scrollTop
    if (mode === 'split' && previewEl) {
      const pct = rawEl.scrollTop / (rawEl.scrollHeight - rawEl.clientHeight)
      if (isFinite(pct)) {
        previewEl.scrollTop = pct * (previewEl.scrollHeight - previewEl.clientHeight)
      }
    }
    requestAnimationFrame(() => { syncing = false })
  }

  function onPreviewScroll() {
    if (mode !== 'split' || !previewEl || !rawEl || syncing) return
    syncing = true
    const pct = previewEl.scrollTop / (previewEl.scrollHeight - previewEl.clientHeight)
    if (isFinite(pct)) {
      rawEl.scrollTop = pct * (rawEl.scrollHeight - rawEl.clientHeight)
    }
    requestAnimationFrame(() => { syncing = false })
  }

  function onInput(e) {
    fileContent.set(e.target.value)
    scheduleAutoSave()
  }

  // Auto-continue list markers when pressing Enter inside a list item.
  // Patterns matched (after optional leading spaces):
  //   - [ ]  - [x]  task lists
  //   -  *  +       unordered
  //   1.  2.  …     ordered
  const LIST_RE = /^(\s*)([-*+] \[[ x]\] |[-*+] |\d+\. )/

  function onKeydown(e) {
    if (e.key !== 'Enter' || e.isComposing) return

    const el = rawEl
    const val = el.value
    const pos = el.selectionStart

    // Find start of the current line
    const lineStart = val.lastIndexOf('\n', pos - 1) + 1
    const line = val.slice(lineStart, pos)

    const m = LIST_RE.exec(line)
    if (!m) return

    e.preventDefault()

    const indent = m[1]
    const marker = m[2]

    // The content after the marker up to the cursor
    const afterMarker = line.slice(indent.length + marker.length)

    if (afterMarker.trim() === '') {
      // Empty item — exit the list by removing the marker
      const newVal = val.slice(0, lineStart) + indent + val.slice(pos)
      el.value = newVal
      const newPos = lineStart + indent.length
      el.setSelectionRange(newPos, newPos)
    } else {
      // Compute next marker
      let nextMarker = marker
      const orderedM = /^(\d+)\. $/.exec(marker)
      if (orderedM) {
        nextMarker = `${parseInt(orderedM[1], 10) + 1}. `
      } else if (/^\s*[-*+] \[[ x]\] $/.test(indent + marker)) {
        // Task list: always continue with unchecked
        nextMarker = marker.replace(/\[[ x]\]/, '[ ]')
      }

      const insert = '\n' + indent + nextMarker
      const newVal = val.slice(0, pos) + insert + val.slice(el.selectionEnd)
      el.value = newVal
      const newPos = pos + insert.length
      el.setSelectionRange(newPos, newPos)
    }

    fileContent.set(el.value)
    scheduleAutoSave()
  }

  function onWikiLink(e) {
    dispatch('wikilink', e.detail)
  }

  // Toggle a task-list checkbox in the raw markdown source
  function onCheckboxToggle(e) {
    const { index, checked } = e.detail
    let count = -1
    const lines = $fileContent.split('\n')
    for (let i = 0; i < lines.length; i++) {
      if (/^\s*[-*+] \[[ x]\]/.test(lines[i])) {
        count++
        if (count === index) {
          lines[i] = lines[i].replace(
            /^(\s*[-*+] \[)[ x](\].*)$/,
            `$1${checked ? 'x' : ' '}$2`
          )
          fileContent.set(lines.join('\n'))
          scheduleAutoSave()
          break
        }
      }
    }
  }
</script>

<div class="editor-wrap">
  <div class="panes" class:split={mode === 'split'}>
    {#if mode === 'edit' || mode === 'split'}
      <div class="raw-wrap" class:split-border={mode === 'split'}>
        {#if $lineNumbers}
          <div class="gutter" bind:this={gutterEl}>
            {#each Array.from({length: lineCount}, (_, i) => i + 1) as n}
              <div class="gutter-line">{n}</div>
            {/each}
          </div>
        {/if}
        <textarea
          bind:this={rawEl}
          class="raw"
          class:full={mode === 'edit'}
          value={$fileContent}
          on:input={onInput}
          on:keydown={onKeydown}
          on:scroll={onRawScroll}
          spellcheck={$spellCheck}
          placeholder="Start writing…"
        ></textarea>
      </div>
    {/if}

    {#if mode === 'preview' || mode === 'split'}
      <div class="preview" class:full={mode === 'preview'} bind:this={previewEl} on:scroll={onPreviewScroll}>
        <Viewer content={$fileContent} {file} on:wikilink={onWikiLink} on:checkbox-toggle={onCheckboxToggle} />
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

  /* ── Panes ───────────────────────────────────────────────────────────────── */
  .panes {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .raw-wrap {
    flex: 1;
    display: flex;
    overflow: hidden;
    min-width: 0;
  }

  .raw-wrap.split-border {
    border-right: 1px solid var(--color-border);
  }

  /* ── Line number gutter ──────────────────────────────────────────────────── */
  .gutter {
    width: 2.8rem;
    overflow: hidden; /* scroll synced via JS */
    padding: 1.25rem 0.5rem 1.25rem 0;
    text-align: right;
    font-family: 'Hack', ui-monospace, 'Cascadia Code', monospace;
    font-size: 0.9rem;
    line-height: 1.7;
    color: var(--color-text-muted);
    background: var(--color-bg);
    user-select: none;
    flex-shrink: 0;
  }

  .gutter-line {
    line-height: 1.7;
    opacity: 0.5;
  }

  /* ── Raw textarea ────────────────────────────────────────────────────────── */
  .raw {
    flex: 1;
    padding: 1.25rem 1.5rem;
    border: none;
    background: var(--color-bg);
    color: var(--color-text);
    font-family: 'Hack', ui-monospace, 'Cascadia Code', monospace;
    font-size: 0.9rem;
    line-height: 1.7;
    resize: none;
    outline: none;
    overflow-y: auto;
    min-width: 0;
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
