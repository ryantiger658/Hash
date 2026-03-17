<script>
  import { get } from 'svelte/store'
  import { createEventDispatcher } from 'svelte'
  import { api } from '../api.js'
  import {
    applyAccentColor, lineNumbers, spellCheck,
    setTheme, getStoredTheme,
  } from '../theme.js'
  import { pollIntervalSecs } from '../../stores/vault.js'

  export let config = {}

  const dispatch = createEventDispatcher()

  // Initialise from LIVE state — what the user is actually seeing right now —
  // not the server config (which may lag if the panel opened before a load completed).
  let accentColor = document.documentElement.style.getPropertyValue('--color-accent-raw')
                    || config.secondary_color
                    || '#aaff00'
  let theme      = getStoredTheme()
  let lnums      = get(lineNumbers)
  let spell      = get(spellCheck)
  let pollSecs   = get(pollIntervalSecs)

  let error      = ''
  let accentTimer = null

  // ── Live preview ────────────────────────────────────────────────────────────
  // These run immediately on every change so the page reflects the new value
  // without waiting for the server round-trip.
  $: applyAccentColor(accentColor)
  $: setTheme(theme)

  // ── Persist helpers ─────────────────────────────────────────────────────────
  async function patch(fields) {
    error = ''
    try {
      await api.postUiConfig(fields)
    } catch (e) {
      error = e.message ?? 'Failed to save'
    }
  }

  // Debounce color saves so we don't fire on every pixel drag.
  function onAccentInput() {
    clearTimeout(accentTimer)
    accentTimer = setTimeout(() => patch({ secondary_color: accentColor }), 600)
  }

  function onThemeChange() {
    patch({ default_theme: theme })
  }

  function onLineNumbersChange() {
    lineNumbers.set(lnums)
    patch({ line_numbers: lnums })
  }

  function onSpellCheckChange() {
    spellCheck.set(spell)
    patch({ spell_check: spell })
  }

  function onPollSecsChange() {
    const v = Math.max(1, Math.round(pollSecs))
    pollSecs = v
    pollIntervalSecs.set(v)             // keeps displayed value correct on re-open
    dispatch('poll-interval-change', { secs: v })  // restarts the active timer
    patch({ poll_interval_secs: v })
  }

  function close() {
    clearTimeout(accentTimer)
    dispatch('close', { default_theme: theme })
  }

  function onBackdropClick(e) {
    if (e.target === e.currentTarget) close()
  }

  function onKeydown(e) {
    if (e.key === 'Escape') close()
  }
</script>

<svelte:window on:keydown={onKeydown} />

<!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
<div class="backdrop" on:click={onBackdropClick}>
  <div class="panel" role="dialog" aria-modal="true" aria-label="Settings">
    <div class="panel-header">
      <h2>Settings</h2>
      <button class="close-btn" on:click={close} title="Close">✕</button>
    </div>

    <div class="panel-body">
      <!-- Accent color -->
      <div class="setting-row">
        <span class="setting-label">Accent color</span>
        <div class="color-row">
          <input type="color" bind:value={accentColor} class="color-swatch"
            on:input={onAccentInput} />
          <input type="text" bind:value={accentColor} class="color-text"
            placeholder="#aaff00"
            pattern="^#[0-9a-fA-F]{6}$"
            spellcheck="false"
            on:input={onAccentInput}
          />
        </div>
      </div>

      <!-- Theme -->
      <div class="setting-row">
        <span class="setting-label">Theme</span>
        <div class="theme-group">
          {#each ['light', 'dark', 'system'] as t}
            <label class="radio-opt" class:selected={theme === t}>
              <input type="radio" bind:group={theme} value={t} on:change={onThemeChange} />
              {t.charAt(0).toUpperCase() + t.slice(1)}
            </label>
          {/each}
        </div>
      </div>

      <!-- Toggles -->
      <label class="setting-row toggle" on:change={onLineNumbersChange}>
        <span class="setting-label">Line numbers</span>
        <input type="checkbox" bind:checked={lnums} class="toggle-input" />
        <span class="toggle-track" class:on={lnums}></span>
      </label>

      <label class="setting-row toggle" on:change={onSpellCheckChange}>
        <span class="setting-label">Spell check</span>
        <input type="checkbox" bind:checked={spell} class="toggle-input" />
        <span class="toggle-track" class:on={spell}></span>
      </label>

      <!-- Sync poll interval -->
      <div class="setting-row">
        <span class="setting-label">Sync interval</span>
        <div class="num-row">
          <input type="number" bind:value={pollSecs} min="1" max="300" step="1"
            class="num-input" on:change={onPollSecsChange} />
          <span class="num-unit">s</span>
        </div>
      </div>

      <p class="static-note">
        <strong>show_hidden_files</strong> requires a server restart to change (set via
        <code>config.toml</code> or <code>HASH_SHOW_HIDDEN_FILES</code>).
      </p>

      {#if error}
        <p class="error">{error}</p>
      {/if}
    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }

  .panel {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: 10px;
    width: min(420px, calc(100vw - 32px));
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0,0,0,0.35);
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 18px 10px;
    border-bottom: 1px solid var(--color-border);
  }

  .panel-header h2 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
    color: var(--color-text);
  }

  .close-btn {
    background: none;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    font-size: 1rem;
    padding: 2px 6px;
    border-radius: 4px;
  }
  .close-btn:hover { background: var(--color-border); color: var(--color-text); }

  .panel-body {
    padding: 14px 18px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
  }

  .setting-label {
    font-size: 0.875rem;
    color: var(--color-text);
    flex-shrink: 0;
  }

  /* Color picker row */
  .color-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .color-swatch {
    width: 32px;
    height: 28px;
    padding: 0;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    cursor: pointer;
    background: none;
  }

  .color-text {
    width: 90px;
    padding: 4px 8px;
    font-size: 0.85rem;
    font-family: inherit;
    background: var(--color-bg);
    color: var(--color-text);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    outline: none;
  }
  .color-text:focus { border-color: var(--color-accent); }

  /* Theme radio group */
  .theme-group {
    display: flex;
    gap: 6px;
  }

  .radio-opt {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 3px 10px;
    font-size: 0.8rem;
    border: 1px solid var(--color-border);
    border-radius: 20px;
    cursor: pointer;
    color: var(--color-text-muted);
    transition: border-color 0.1s, color 0.1s;
  }
  .radio-opt input { display: none; }
  .radio-opt.selected {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  /* Toggle switch */
  .toggle { cursor: pointer; }
  .toggle-input { display: none; }

  .toggle-track {
    position: relative;
    width: 36px;
    height: 20px;
    background: var(--color-border);
    border-radius: 10px;
    flex-shrink: 0;
    transition: background 0.2s;
  }
  .toggle-track.on { background: var(--color-accent); }
  .toggle-track::after {
    content: '';
    position: absolute;
    top: 3px;
    left: 3px;
    width: 14px;
    height: 14px;
    background: #fff;
    border-radius: 50%;
    transition: transform 0.2s;
  }
  .toggle-track.on::after { transform: translateX(16px); }

  /* Number input row */
  .num-row {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .num-input {
    width: 72px;
    padding: 4px 8px;
    font-size: 0.85rem;
    font-family: inherit;
    background: var(--color-bg);
    color: var(--color-text);
    border: 1px solid var(--color-border);
    border-radius: 4px;
    outline: none;
    text-align: right;
  }
  .num-input:focus { border-color: var(--color-accent); }

  .num-unit {
    font-size: 0.8rem;
    color: var(--color-text-muted);
  }

  .static-note {
    font-size: 0.78rem;
    color: var(--color-text-muted);
    margin: 0;
    padding: 8px 10px;
    background: var(--color-border);
    border-radius: 6px;
    line-height: 1.5;
  }
  .static-note code {
    font-size: 0.78rem;
    background: none;
  }

  .error {
    font-size: 0.8rem;
    color: #f87171;
    margin: 0;
  }
</style>
