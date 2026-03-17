/**
 * Theme management for #ash.
 *
 * - Accent color is fetched from the server's /api/ui-config on startup.
 * - Light/dark preference is stored in localStorage, with "system" as the default.
 * - Both are applied as CSS custom properties on <html>.
 */
import { writable, get } from 'svelte/store'
import { api } from './api.js'

export const THEMES = ['light', 'dark', 'system']

/** Reactive store tracking the active theme — updated by setTheme(). */
export const activeTheme = writable(localStorage.getItem('hash-theme') ?? 'system')

/** Whether to show line numbers in the editor (server-configured). */
export const lineNumbers = writable(false)

/** Whether to enable browser spell-check in the editor (server-configured). */
export const spellCheck = writable(false)

/** Short-lived session token for vault-asset image URLs (not the API key). */
export const imageToken = writable('')

/** Fetch a new session token and store it. Call after successful login. */
export async function refreshImageToken() {
  try {
    const { token } = await api.createSession()
    imageToken.set(token)
  } catch {
    // Non-fatal — images just won't load if unauthenticated
  }
}

/** Fetch UI config from the server, apply settings, and return the raw config object.
 *  The caller can use `poll_interval_secs` etc. to configure polling. */
export async function loadServerTheme() {
  try {
    const res = await fetch('/api/ui-config')
    if (!res.ok) return null
    const cfg = await res.json()
    const { secondary_color, default_theme, line_numbers, spell_check } = cfg

    applyAccentColor(secondary_color)
    lineNumbers.set(!!line_numbers)
    spellCheck.set(!!spell_check)

    // Server is authoritative for theme. Apply it and update localStorage so
    // initTheme() prevents a flash on next page load.
    setTheme(default_theme)
    return cfg
  } catch {
    // Server unreachable (e.g. desktop offline mode) — fall back to stored or system.
    return null
  }
}

/** Apply a CSS hex accent color as --color-accent-raw (CSS derives contextual shades). */
export function applyAccentColor(hex) {
  if (!hex || !/^#[0-9a-fA-F]{6}$/.test(hex)) return
  const el = document.documentElement
  // Set the raw value — CSS computes --color-accent (darkened in light mode) from this.
  el.style.setProperty('--color-accent-raw', hex)
  el.style.setProperty('--color-accent-dim', hex + '26') // ~15% opacity background tint
}

/** Set the active theme: 'light', 'dark', or 'system'. Persists to localStorage. */
export function setTheme(theme) {
  if (!THEMES.includes(theme)) return
  localStorage.setItem('hash-theme', theme)
  activeTheme.set(theme)
  applyTheme(theme)
}

/** Read stored theme preference (or fall back to 'system') without persisting. */
export function getStoredTheme() {
  return localStorage.getItem('hash-theme') ?? 'system'
}

/** Apply a theme value to the document immediately. */
function applyTheme(theme) {
  const html = document.documentElement
  if (theme === 'system') {
    html.removeAttribute('data-theme')
  } else {
    html.setAttribute('data-theme', theme)
  }
}

/** Initialize theme on page load — call once before first render. */
export function initTheme() {
  applyTheme(getStoredTheme())
}
