/**
 * Theme management for #ash.
 *
 * - Accent color is fetched from the server's /api/ui-config on startup.
 * - Light/dark preference is stored in localStorage, with "system" as the default.
 * - Both are applied as CSS custom properties on <html>.
 */

export const THEMES = ['light', 'dark', 'system']

/** Fetch UI config from the server and apply the accent color. */
export async function loadServerTheme() {
  try {
    const res = await fetch('/api/ui-config')
    if (!res.ok) return
    const { secondary_color, default_theme } = await res.json()

    applyAccentColor(secondary_color)

    // Only apply the server's default_theme if the user hasn't saved a preference.
    if (!localStorage.getItem('hash-theme')) {
      setTheme(default_theme)
    }
  } catch {
    // Server unreachable (e.g. desktop offline mode) — fall back to stored or system.
  }
}

/** Apply a CSS hex accent color as --color-accent and derived shades. */
export function applyAccentColor(hex) {
  if (!hex || !/^#[0-9a-fA-F]{6}$/.test(hex)) return
  const el = document.documentElement
  el.style.setProperty('--color-accent', hex)
  el.style.setProperty('--color-accent-dim', hex + 'aa') // 67% opacity variant
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
