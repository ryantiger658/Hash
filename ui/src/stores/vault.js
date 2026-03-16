/**
 * Vault state store for #ash.
 * Holds the file list, selected file, editor content, and save status.
 */
import { writable, derived, get } from 'svelte/store'
import { api, AuthError } from '../lib/api.js'

// ── Raw stores ────────────────────────────────────────────────────────────────

export const files = writable([])           // FileEntry[] from server
export const selectedPath = writable(null)  // vault-relative path of open file
export const fileContent = writable('')     // current editor content
export const savedContent = writable('')    // content as last saved to server
export const isLoading = writable(false)
export const saveStatus = writable('idle')  // 'idle' | 'saving' | 'saved' | 'error'

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
    files.set(result)
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
  const text = await api.getFile(path)
  fileContent.set(text)
  savedContent.set(text)
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
 * Delete every file whose path starts with `folderPath/`.
 * Clears the editor if the open file was inside the folder.
 */
export async function deleteFolder(folderPath) {
  const prefix = folderPath.endsWith('/') ? folderPath : `${folderPath}/`
  const toDelete = get(files).filter(f => f.path.startsWith(prefix))
  await Promise.all(toDelete.map(f => api.deleteFile(f.path)))
  if (get(selectedPath)?.startsWith(prefix)) {
    selectedPath.set(null)
    fileContent.set('')
    savedContent.set('')
  }
  await loadVault()
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
        node = { name, path, isDir: !isLast, children: isLast ? undefined : [] }
        siblings.push(node)
        // Directories first, then files, both alphabetical
        siblings.sort((a, b) => {
          if (a.isDir !== b.isDir) return a.isDir ? -1 : 1
          return a.name.localeCompare(b.name)
        })
      }
      if (!isLast) siblings = node.children
    }
  }

  return root
}
