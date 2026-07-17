<script>
  import { createEventDispatcher, onMount } from 'svelte'
  import { saveApiKey, saveServerUrl, getServerUrl, api } from '../api.js'

  const dispatch = createEventDispatcher()

  // In the Tauri desktop app window.__TAURI__ is defined — we need the user to
  // supply the server URL because the webview can't use relative /api/... paths.
  const isDesktop = typeof window !== 'undefined' && !!window.__TAURI__

  let serverUrl = getServerUrl()
  let apiKey = ''
  let error = ''
  let loading = false
  let inputEl
  let oidcEnabled = false

  onMount(async () => {
    if (!isDesktop) {
      try {
        const status = await api.authStatus()
        oidcEnabled = !!status.oidc_enabled
        if (status.authenticated) {
          dispatch('login')
          return
        }
      } catch { /* API-key form remains available */ }
    }
    // Pre-fill server URL from sync config if not already stored
    if (isDesktop && !serverUrl) {
      try {
        const cfg = await window.__TAURI__.core.invoke('get_config')
        if (cfg?.server_url) serverUrl = cfg.server_url
      } catch { /* no config yet */ }
    }
    inputEl?.focus()
  })

  function oidcLogin() {
    window.location.assign('/api/auth/oidc/login')
  }

  async function submit() {
    error = ''
    if (!apiKey.trim()) { error = 'Please enter your API key.'; return }
    if (isDesktop && !serverUrl.trim()) { error = 'Please enter your server URL.'; return }

    loading = true
    try {
      const base = isDesktop ? serverUrl.trim().replace(/\/+$/, '') : ''
      const res = await fetch(`${base}/api/files`, {
        headers: { Authorization: `Bearer ${apiKey.trim()}` },
      })
      if (res.status === 401) { error = 'Invalid API key.'; return }
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      if (isDesktop) saveServerUrl(serverUrl)
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

    {#if oidcEnabled && !isDesktop}
      <button class="oidc-btn" type="button" on:click={oidcLogin}>Sign in with SSO</button>
      <div class="divider"><span>or use an API key</span></div>
    {/if}

    {#if isDesktop}
      <label for="serverurl">Server URL</label>
      <input
        id="serverurl"
        type="url"
        bind:value={serverUrl}
        placeholder="http://192.168.1.100:3535"
        autocomplete="url"
      />
    {/if}

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

  .oidc-btn {
    margin-top: 0.25rem;
    border: 1px solid var(--color-accent);
    background: var(--color-surface);
    color: var(--color-text);
  }

  .oidc-btn:hover,
  .oidc-btn:focus-visible {
    background: color-mix(in srgb, var(--color-accent) 14%, var(--color-surface));
    outline: none;
  }

  .divider { display: flex; align-items: center; gap: 0.6rem; color: var(--color-text-muted); font-size: 0.72rem; }
  .divider::before, .divider::after { content: ''; height: 1px; background: var(--color-border); flex: 1; }
</style>
