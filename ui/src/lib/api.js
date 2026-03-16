/**
 * #ash API client.
 * All requests include the stored API key as a Bearer token.
 */

const API_KEY_STORAGE = 'hash-api-key'

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

  const res = await fetch(`/api${path}`, { method, headers, body: bodyToSend })

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

  /** GET /api/search?q= — full-text search across the vault */
  search: (q) => request('GET', `/search?q=${encodeURIComponent(q)}`).then(r => r.json()),

  /** GET /api/ui-config — public, returns accent color + default theme */
  uiConfig: () => fetch('/api/ui-config').then(r => r.json()),
}
