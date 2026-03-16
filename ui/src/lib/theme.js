/**
 * Theme management for #ash.
 *
 * - Accent color is fetched from the server's /api/ui-config on startup.
 * - Light/dark preference is stored in localStorage, with "system" as the default.
 * - Both are applied as CSS custom properties on <html>.
 */
import { writable } from 'svelte/store'

export const THEMES = ['light', 'dark', 'system']

/** Whether editor mode buttons should show text labels (server-configured). */
export const editorLabels = writable(false)

/** Whether to show line numbers in the editor (server-configured). */
export const lineNumbers = writable(false)

/** Whether to enable browser spell-check in the editor (server-configured). */
export const spellCheck = writable(false)

/** Fetch UI config from the server and apply the accent color. */
export async function loadServerTheme() {
  try {
    const res = await fetch('/api/ui-config')
    if (!res.ok) return
    const { secondary_color, default_theme, editor_labels, line_numbers, spell_check } = await res.json()

    applyAccentColor(secondary_color)
    editorLabels.set(!!editor_labels)
    lineNumbers.set(!!line_numbers)
    spellCheck.set(!!spell_check)

    // Only apply the server's default_theme if the user hasn't saved a preference.
    if (!localStorage.getItem('hash-theme')) {
      setTheme(default_theme)
    }
  } catch {
    // Server unreachable (e.g. desktop offline mode) — fall back to stored or system.
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
