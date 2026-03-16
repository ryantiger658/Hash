<script>
  import { createEventDispatcher } from 'svelte'

  export let show = false

  const dispatch = createEventDispatcher()

  let name = ''
  let error = ''
  let input

  $: if (show && input) {
    setTimeout(() => input?.focus(), 50)
  }

  function submit() {
    const trimmed = name.trim()
    if (!trimmed) {
      error = 'Please enter a name.'
      return
    }
    // Sanitize: strip leading slashes and disallow traversal
    if (trimmed.includes('..')) {
      error = 'Name cannot contain ".."'
      return
    }
    dispatch('create', trimmed)
    close()
  }

  function close() {
    show = false
    name = ''
    error = ''
  }
</script>

{#if show}
  <!-- svelte-ignore a11y-click-events-have-key-events a11y-no-static-element-interactions -->
  <div class="overlay" on:click|self={close}>
    <div class="modal" role="dialog" aria-modal="true" aria-label="New note">
      <h2>New Note</h2>
      <p class="hint">Enter a name or path, e.g. <code>note.md</code> or <code>folder/note.md</code></p>

      <form on:submit|preventDefault={submit}>
        <input
          bind:this={input}
          bind:value={name}
          placeholder="note.md"
          autocomplete="off"
          spellcheck="false"
        />
        {#if error}
          <p class="error">{error}</p>
        {/if}
        <div class="actions">
          <button type="button" class="secondary" on:click={close}>Cancel</button>
          <button type="submit">Create</button>
        </div>
      </form>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }

  .modal {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 10px;
    padding: 1.5rem;
    width: 100%;
    max-width: 380px;
    box-shadow: 0 16px 48px rgba(0,0,0,0.3);
  }

  h2 {
    font-size: 1.1rem;
    margin-bottom: 0.25rem;
  }

  .hint {
    font-size: 0.8rem;
    color: var(--color-text-muted);
    margin-bottom: 1rem;
  }

  .hint code {
    background: var(--color-border);
    padding: 0.1em 0.3em;
    border-radius: 3px;
    font-size: 0.85em;
  }

  input {
    width: 100%;
    padding: 0.6rem 0.75rem;
    border: 1px solid var(--color-border);
    border-radius: 6px;
    background: var(--color-bg);
    color: var(--color-text);
    font-size: 0.9rem;
    outline: none;
    box-sizing: border-box;
    transition: border-color 0.15s;
  }

  input:focus {
    border-color: var(--color-accent);
  }

  .error {
    font-size: 0.82rem;
    color: #f87171;
    margin-top: 0.35rem;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 0.5rem;
    margin-top: 1rem;
  }

  button {
    padding: 0.5rem 1rem;
    border-radius: 6px;
    border: none;
    font-size: 0.875rem;
    font-weight: 500;
    cursor: pointer;
    transition: opacity 0.15s;
  }

  button[type="submit"] {
    background: var(--color-accent);
    color: #000;
  }

  button.secondary {
    background: var(--color-border);
    color: var(--color-text);
  }
</style>
