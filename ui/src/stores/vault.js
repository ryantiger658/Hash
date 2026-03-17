/**
 * Vault state store for #ash.
 * Holds the file list, selected file, editor content, and save status.
 */
import { writable, derived, get } from 'svelte/store'
import { api } from '../lib/api.js'
import { parseFrontmatter, normalizeArray } from '../lib/frontmatter.js'

// ── Raw stores ────────────────────────────────────────────────────────────────

export const files = writable([])           // FileEntry[] from server
export const selectedPath = writable(null)  // vault-relative path of open file
export const fileContent = writable('')     // current editor content
export const savedContent = writable('')    // content as last saved to server
export const isLoading = writable(false)
export const saveStatus = writable('idle')  // 'idle' | 'saving' | 'saved' | 'error'

/**
 * Active poll interval in seconds. Updated from server config on login and
 * whenever the user changes it in the settings panel. App.svelte watches this
 * reactively to restart the polling timer with the correct interval.
 */
export const pollIntervalSecs = writable(10)

/**
 * True when the currently open file has been modified on the server while the
 * editor has unsaved local changes. Cleared on reload or discard.
 */
export const remoteChangeAvailable = writable(false)

/**
 * Maps alias (lowercase) → vault path.
 * Built up as files are opened — not complete until each file is visited at least once.
 */
export const aliasMap = writable(new Map())

// ── Derived ───────────────────────────────────────────────────────────────────

/** True when the editor has unsaved changes. */
export const isDirty = derived(
  [fileContent, savedContent],
  ([$fc, $sc]) => $fc !== $sc
)

/** The FileEntry for the currently open file, or null. */
export const selectedFile = derived(
  [files, selectedPath],
  ([$files, $path]) => $files.find(f => f.path === $path) ?? null
)

/**
 * File tree built from the flat file list.
 * Each node: { name, path, isDir, children? }
 */
export const fileTree = derived(files, ($files) => buildTree($files))

// ── Actions ───────────────────────────────────────────────────────────────────

/** Fetch the full file list from the server. */
export async function loadVault() {
  isLoading.set(true)
  try {
    const result = await api.listFiles()
    files.set(Array.isArray(result) ? result : [])
  } finally {
    isLoading.set(false)
  }
}

const MONTHS = ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec']

/** Open today's journal entry, creating it if it doesn't exist yet. */
export async function openTodayJournal() {
  const today = new Date()
  const yyyy  = today.getFullYear()
  const mm    = String(today.getMonth() + 1).padStart(2, '0')
  const dd    = String(today.getDate()).padStart(2, '0')
  const mon   = MONTHS[today.getMonth()]
  const datestamp = `${mm}-${dd}-${yyyy}`
  const path = `journal/${yyyy}/${mon}/${datestamp}.md`

  const list = get(files)
  if (list.find(f => f.path === path)) {
    await selectFile(path)
  } else {
    const heading = `# ${datestamp}\n\n`
    await api.putFile(path, heading)
    await loadVault()
    await selectFile(path)
  }
}

/** Open a file: fetch its content and set it as the active editor document. */
export async function selectFile(path) {
  // Silently delete the previous journal entry if it's still blank
  const prevPath = get(selectedPath)
  if (prevPath && prevPath !== path && isBlankJournal(prevPath, get(savedContent))) {
    api.deleteFile(prevPath).then(() => loadVault()).catch(() => {})
  }

  selectedPath.set(path)

  // Non-markdown assets (images, attachments) are shown in the AssetViewer —
  // no text content to fetch or edit.
  if (!path.endsWith('.md')) {
    fileContent.set('')
    savedContent.set('')
    return
  }

  const text = await api.getFile(path)
  fileContent.set(text)
  savedContent.set(text)

  // Cache any aliases declared in frontmatter for wiki-link resolution
  const { meta } = parseFrontmatter(text)
  const aliases = normalizeArray(meta.aliases ?? meta.alias)
  if (aliases.length) {
    aliasMap.update(m => {
      const next = new Map(m)
      for (const a of aliases) next.set(a.toLowerCase(), path)
      return next
    })
  }
}

/**
 * Upload a FileList into the vault under targetFolder.
 * Each file is read as binary and PUT to /api/files/<folder>/<filename>.
 * Reloads the vault file list when all uploads settle.
 */
export async function uploadFiles(fileList, targetFolder = '') {
  const uploads = Array.from(fileList).map(async (file) => {
    const prefix = targetFolder ? `${targetFolder}/` : ''
    const path = prefix + file.name
    const buf = await file.arrayBuffer()
    await api.putFile(path, new Uint8Array(buf))
  })
  await Promise.allSettled(uploads)
  await loadVault()
}

/** True if `path` is a journal file whose content is only the auto-generated heading. */
function isBlankJournal(path, content) {
  if (!/^journal\/\d{4}\/[A-Za-z]{3}\/\d{2}-\d{2}-\d{4}\.md$/.test(path)) return false
  const filename = path.split('/').pop().replace('.md', '')
  const trimmed = content.trim()
  return trimmed === '' || trimmed === `# ${filename}`
}

/** Create a new empty file at the given vault-relative path. */
export async function createFile(path) {
  // Ensure .md extension
  const fullPath = path.endsWith('.md') ? path : `${path}.md`
  await api.putFile(fullPath, '')
  await loadVault()
  await selectFile(fullPath)
}

/**
 * Recursively delete a folder and all its contents.
 * Clears the editor if the open file was inside the folder.
 */
export async function deleteFolder(folderPath) {
  const prefix = folderPath.endsWith('/') ? folderPath : `${folderPath}/`
  await api.deleteDir(folderPath)
  if (get(selectedPath)?.startsWith(prefix)) {
    selectedPath.set(null)
    fileContent.set('')
    savedContent.set('')
  }
  await loadVault()
}

/** Delete any file by path. Clears the editor if the deleted file was open. */
export async function deleteFile(path) {
  if (!path) return
  await api.deleteFile(path)
  if (get(selectedPath) === path) {
    selectedPath.set(null)
    fileContent.set('')
    savedContent.set('')
  }
  await loadVault()
}

/**
 * Rename a file or folder. If the renamed file was open, re-opens it at the new path.
 */
export async function renameItem(fromPath, toPath) {
  const wasOpen = get(selectedPath) === fromPath
  await api.renameFile(fromPath, toPath)
  if (wasOpen) {
    selectedPath.set(null)
    fileContent.set('')
    savedContent.set('')
  }
  await loadVault()
  if (wasOpen) {
    const newFile = get(files).find(f => f.path === toPath)
    if (newFile && !newFile.isDir) await selectFile(toPath)
  }
}

/** Delete a file. Clears the editor if the deleted file was open. */
export async function deleteCurrentFile() {
  const path = get(selectedPath)
  if (!path) return
  await api.deleteFile(path)
  selectedPath.set(null)
  fileContent.set('')
  savedContent.set('')
  await loadVault()
}

/** Save the current editor content to the server immediately. */
export async function saveCurrentFile() {
  const path = get(selectedPath)
  const content = get(fileContent)
  if (!path || !get(isDirty)) return
  saveStatus.set('saving')
  try {
    await api.putFile(path, content)
    savedContent.set(content)
    saveStatus.set('saved')
    loadVault() // refresh checksums in file list
    setTimeout(() => saveStatus.set('idle'), 2000)
  } catch (e) {
    console.error('Save failed', e)
    saveStatus.set('error')
    setTimeout(() => saveStatus.set('idle'), 3000)
  }
}

// ── Background polling ────────────────────────────────────────────────────────

let pollTimer = null

/**
 * Compare the server's file list against the local store.
 * - Updates the sidebar if any entries changed.
 * - Silently reloads the open file if its checksum changed and there are no
 *   unsaved edits.
 * - Sets remoteChangeAvailable if the open file changed while the editor is
 *   dirty (so the user can choose to reload or keep their edits).
 */
export async function pollVault() {
  try {
    const result = await api.listFiles()
    const newFiles = Array.isArray(result) ? result : []
    const current = get(files)

    const currentMap = new Map(current.map(f => [f.path, f.checksum]))
    const newMap = new Map(newFiles.map(f => [f.path, f.checksum]))

    // Cheap early exit — nothing changed at all.
    const listChanged =
      newFiles.length !== current.length ||
      newFiles.some(f => currentMap.get(f.path) !== f.checksum) ||
      current.some(f => !newMap.has(f.path))

    if (!listChanged) return

    files.set(newFiles)

    const openPath = get(selectedPath)
    if (!openPath) return

    const oldChecksum = currentMap.get(openPath)
    const newChecksum = newMap.get(openPath)

    // Open file unchanged — nothing more to do.
    if (oldChecksum === newChecksum) return

    if (newChecksum === undefined) {
      // File was deleted on the server — clear the editor.
      selectedPath.set(null)
      fileContent.set('')
      savedContent.set('')
      remoteChangeAvailable.set(false)
      return
    }

    if (get(isDirty)) {
      // Unsaved local changes conflict with the remote update — let the user decide.
      remoteChangeAvailable.set(true)
    } else {
      // No local changes — reload silently.
      const text = await api.getFile(openPath)
      fileContent.set(text)
      savedContent.set(text)
      remoteChangeAvailable.set(false)
    }
  } catch {
    // Network error or 401 — non-fatal, skip this poll cycle.
  }
}

/** Reload the open file from the server, discarding any unsaved local changes. */
export async function acceptRemoteChange() {
  const path = get(selectedPath)
  if (!path) return
  const text = await api.getFile(path)
  fileContent.set(text)
  savedContent.set(text)
  remoteChangeAvailable.set(false)
}

/** Start polling the vault every `intervalMs` milliseconds. */
export function startPolling(intervalMs = 10_000) {
  stopPolling()
  pollTimer = setInterval(pollVault, intervalMs)
}

/** Stop background polling. */
export function stopPolling() {
  if (pollTimer != null) {
    clearInterval(pollTimer)
    pollTimer = null
  }
}

// ── Open-file fast poll ───────────────────────────────────────────────────────

let openFilePollTimer = null

/**
 * Check only the currently open file against the server.
 * Uses the lightweight `/api/checksum/*path` endpoint — no full vault scan.
 * Updates the files store entry so the next full pollVault() sees it as fresh.
 */
export async function pollOpenFile() {
  const path = get(selectedPath)
  if (!path) return
  try {
    const { checksum: newChecksum, modified: newModified } = await api.fileChecksum(path)
    const current = get(files)
    const entry = current.find(f => f.path === path)
    if (!entry || entry.checksum === newChecksum) return

    // Update the store so the next full poll doesn't double-trigger.
    files.update(list =>
      list.map(f => (f.path === path ? { ...f, checksum: newChecksum, modified: newModified } : f))
    )

    if (get(isDirty)) {
      remoteChangeAvailable.set(true)
    } else {
      const text = await api.getFile(path)
      fileContent.set(text)
      savedContent.set(text)
      remoteChangeAvailable.set(false)
    }
  } catch {
    // Non-fatal.
  }
}

/** Start a fast per-open-file poll at `intervalMs` (default 2 s). */
export function startOpenFilePoll(intervalMs = 2_000) {
  stopOpenFilePoll()
  openFilePollTimer = setInterval(pollOpenFile, intervalMs)
}

/** Stop the open-file poll. */
export function stopOpenFilePoll() {
  if (openFilePollTimer != null) {
    clearInterval(openFilePollTimer)
    openFilePollTimer = null
  }
}

// ── Auto-save ─────────────────────────────────────────────────────────────────

let autoSaveTimer = null

/** Schedule an auto-save 1.5s after the last edit. */
export function scheduleAutoSave() {
  clearTimeout(autoSaveTimer)
  autoSaveTimer = setTimeout(saveCurrentFile, 1500)
}

// ── Tree builder ──────────────────────────────────────────────────────────────

function buildTree(files) {
  const root = []

  for (const file of [...files].sort((a, b) => a.path.localeCompare(b.path))) {
    const parts = file.path.split('/')
    let siblings = root

    for (let i = 0; i < parts.length; i++) {
      const name = parts[i]
      const path = parts.slice(0, i + 1).join('/')
      const isLast = i === parts.length - 1

      let node = siblings.find(n => n.name === name)
      if (!node) {
        const isDir = !isLast || !!file.isDir
        node = { name, path, isDir, children: isDir ? [] : undefined }
        siblings.push(node)
        // journal dir pinned first, then other dirs, then files — all alphabetical within group
        siblings.sort((a, b) => {
          if (a.isDir !== b.isDir) return a.isDir ? -1 : 1
          if (a.name === 'journal' && a.isDir) return -1
          if (b.name === 'journal' && b.isDir) return 1
          return a.name.localeCompare(b.name)
        })
      }
      if (!isLast) siblings = node.children
    }
  }

  return root
}
