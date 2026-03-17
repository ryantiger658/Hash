/**
 * #ash API client.
 * All requests include the stored API key as a Bearer token.
 *
 * In the browser (served by Axum) all paths are relative — /api/...
 * In the Tauri desktop app the webview loads from the app bundle, so
 * relative paths don't reach the server.  We store the server origin in
 * localStorage and prepend it to every request when present.
 */

const API_KEY_STORAGE    = 'hash-api-key'
const SERVER_URL_STORAGE = 'hash-server-url'

export function getApiKey() {
  return localStorage.getItem(API_KEY_STORAGE) ?? ''
}

export function saveApiKey(key) {
  localStorage.setItem(API_KEY_STORAGE, key.trim())
}

export function clearApiKey() {
  localStorage.removeItem(API_KEY_STORAGE)
}

export function hasApiKey() {
  return !!getApiKey()
}

/** Returns the stored server origin (e.g. "http://192.168.1.10:3535"), or '' in browser mode. */
export function getServerUrl() {
  return localStorage.getItem(SERVER_URL_STORAGE) ?? ''
}

export function saveServerUrl(url) {
  // Strip trailing slash so we can always do `${base}/api/...`
  localStorage.setItem(SERVER_URL_STORAGE, url.trim().replace(/\/+$/, ''))
}

export function clearServerUrl() {
  localStorage.removeItem(SERVER_URL_STORAGE)
}

// Encode a vault-relative path for use in a URL.
// Encodes each segment individually to preserve forward slashes.
function encodePath(path) {
  return path.split('/').map(encodeURIComponent).join('/')
}

export class AuthError extends Error {}

async function request(method, path, body = null) {
  const headers = { Authorization: `Bearer ${getApiKey()}` }

  let bodyToSend = null
  if (typeof body === 'string') {
    headers['Content-Type'] = 'text/plain; charset=utf-8'
    bodyToSend = body
  } else if (body instanceof Uint8Array) {
    headers['Content-Type'] = 'application/octet-stream'
    bodyToSend = body
  } else if (body !== null) {
    headers['Content-Type'] = 'application/json'
    bodyToSend = JSON.stringify(body)
  }

  const base = getServerUrl() // '' in browser, 'http://host:port' in desktop
  const res = await fetch(`${base}/api${path}`, { method, headers, body: bodyToSend })

  if (res.status === 401) throw new AuthError('Invalid API key')
  if (!res.ok) throw new Error(`HTTP ${res.status}: ${await res.text()}`)
  return res
}

export const api = {
  /** GET /api/files — list all vault files with metadata */
  listFiles: () => request('GET', '/files').then(r => r.json()),

  /** GET /api/files/{path} — fetch a single file as text */
  getFile: (path) => request('GET', `/files/${encodePath(path)}`).then(r => r.text()),

  /** PUT /api/files/{path} — create or overwrite a file */
  putFile: (path, content) => request('PUT', `/files/${encodePath(path)}`, content),

  /** DELETE /api/files/{path} — delete a file */
  deleteFile: (path) => request('DELETE', `/files/${encodePath(path)}`),

  /** DELETE /api/dirs/{path} — recursively delete a directory */
  deleteDir: (path) => request('DELETE', `/dirs/${encodePath(path)}`),

  /** POST /api/files/rename — rename or move a file or directory */
  renameFile: (from, to) => request('POST', '/files/rename', { from, to }),

  /** GET /api/search?q= — full-text search across the vault */
  search: (q) => request('GET', `/search?q=${encodeURIComponent(q)}`).then(r => r.json()),

  /** POST /api/auth/session — exchange API key for a short-lived image session token */
  createSession: () => request('POST', '/auth/session').then(r => r.json()),

  /** POST /api/ui-config — update mutable UI settings (partial patch) */
  postUiConfig: (patch) => request('POST', '/ui-config', patch).then(r => r.json()),

  /** GET /api/ui-config — public, returns accent color + default theme */
  uiConfig: () => fetch(`${getServerUrl()}/api/ui-config`).then(r => r.json()),

  /** GET /api/checksum/{path} — fast single-file change detection (checksum + modified) */
  fileChecksum: (path) => request('GET', `/checksum/${encodePath(path)}`).then(r => r.json()),
}
