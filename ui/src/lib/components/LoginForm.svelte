<script>
  import { createEventDispatcher, onMount } from 'svelte'
  import { saveApiKey } from '../api.js'

  const dispatch = createEventDispatcher()

  let apiKey = ''
  let error = ''
  let loading = false
  let inputEl

  onMount(() => inputEl?.focus())

  async function submit() {
    error = ''
    if (!apiKey.trim()) {
      error = 'Please enter your API key.'
      return
    }
    loading = true
    try {
      // Test the key against a real API endpoint
      const res = await fetch('/api/files', {
        headers: { Authorization: `Bearer ${apiKey.trim()}` },
      })
      if (res.status === 401) {
        error = 'Invalid API key.'
        return
      }
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      saveApiKey(apiKey)
      dispatch('login')
    } catch (e) {
      error = e.message === 'Invalid API key.' ? e.message : 'Could not reach the server.'
    } finally {
      loading = false
    }
  }
</script>

<div class="login-wrap">
  <form class="login-card" on:submit|preventDefault={submit}>
    <h1 class="logo">#ash</h1>
    <p class="tagline">Self-hosted markdown knowledge base</p>

    <label for="apikey">API Key</label>
    <input
      bind:this={inputEl}
      id="apikey"
      type="password"
      bind:value={apiKey}
      placeholder="Enter your API key"
      autocomplete="current-password"
    />

    {#if error}
      <p class="error">{error}</p>
    {/if}

    <button type="submit" disabled={loading}>
      {loading ? 'Connecting…' : 'Connect'}
    </button>
  </form>
</div>

<style>
  .login-wrap {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
    background: var(--color-bg);
  }

  .login-card {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    width: 100%;
    max-width: 360px;
    padding: 2.5rem;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: 12px;
  }

  .logo {
    font-size: 2rem;
    font-weight: 800;
    color: var(--color-accent);
    text-align: center;
    margin-bottom: 0;
  }

  .tagline {
    color: var(--color-text-muted);
    font-size: 0.85rem;
    text-align: center;
    margin-bottom: 0.5rem;
  }

  label {
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--color-text-muted);
  }

  input {
    padding: 0.6rem 0.75rem;
    border-radius: 6px;
    border: 1px solid var(--color-border);
    background: var(--color-bg);
    color: var(--color-text);
    font-size: 0.95rem;
    outline: none;
    transition: border-color 0.15s;
  }

  input:focus {
    border-color: var(--color-accent);
  }

  button {
    margin-top: 0.5rem;
    padding: 0.65rem;
    border-radius: 6px;
    border: none;
    background: var(--color-accent);
    color: #fff;
    font-size: 0.95rem;
    font-weight: 600;
    cursor: pointer;
    transition: opacity 0.15s;
  }

  button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .error {
    font-size: 0.85rem;
    color: #f87171;
  }
</style>
