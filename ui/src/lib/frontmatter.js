/**
 * Frontmatter parser for #ash (Obsidian-compatible).
 *
 * Handles the YAML subset Obsidian actually writes:
 *   - Scalar strings, booleans, numbers
 *   - Inline arrays: tags: [rust, systems]
 *   - Block arrays:  tags:\n  - rust\n  - systems
 *   - Quoted strings
 */

// Matches a leading ---...--- block (with optional \r\n line endings)
const FM_RE = /^---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)/

/**
 * Extract and parse YAML frontmatter from a markdown string.
 * Returns the parsed metadata and the body text with frontmatter removed.
 * Never throws — malformed YAML returns empty meta and the original body.
 *
 * @param {string} content
 * @returns {{ meta: Record<string, any>, body: string }}
 */
export function parseFrontmatter(content) {
  const src = content ?? ''
  const match = FM_RE.exec(src)
  if (!match) return { meta: {}, body: src }

  const yamlSrc = match[1]
  const body = src.slice(match[0].length)

  try {
    const meta = parseYaml(yamlSrc)
    return { meta, body }
  } catch (_) {
    return { meta: {}, body }
  }
}

/**
 * Return a normalized string array from a frontmatter tag value.
 * Accepts: string, string[], or null/undefined.
 * Obsidian also allows space-separated tags in a single string ("rust systems").
 *
 * @param {any} raw
 * @returns {string[]}
 */
export function normalizeTags(raw) {
  if (!raw) return []
  const arr = Array.isArray(raw) ? raw : String(raw).split(/[\s,]+/)
  return arr
    .map(t => String(t ?? '').trim().replace(/^#/, ''))
    .filter(Boolean)
}

/**
 * Return a normalized string array from any frontmatter array-or-string field.
 * @param {any} raw
 * @returns {string[]}
 */
export function normalizeArray(raw) {
  if (!raw) return []
  if (Array.isArray(raw)) return raw.map(String).filter(Boolean)
  return [String(raw)].filter(Boolean)
}

// ── Minimal YAML parser ───────────────────────────────────────────────────────

function parseYaml(src) {
  const result = {}
  const lines = src.split(/\r?\n/)
  let i = 0

  while (i < lines.length) {
    const line = lines[i]

    // Skip blank lines and comments
    if (!line.trim() || line.trimStart().startsWith('#')) {
      i++
      continue
    }

    // Key: value  (key must start at column 0)
    const kv = /^([a-zA-Z_][a-zA-Z0-9_ -]*):\s*(.*)$/.exec(line)
    if (!kv) { i++; continue }

    const key = kv[1].trim()
    const raw = kv[2].trim()

    if (raw === '' || raw === '|' || raw === '>') {
      // Block sequence — look ahead for "  - item" lines
      const items = []
      i++
      while (i < lines.length && /^[ \t]+-[ \t]/.test(lines[i])) {
        const item = lines[i].replace(/^[ \t]+-[ \t]/, '').trim()
        items.push(stripQuotes(item))
        i++
      }
      result[key] = items
    } else if (raw.startsWith('[') && raw.endsWith(']')) {
      // Inline array: [a, b, c]
      result[key] = raw
        .slice(1, -1)
        .split(',')
        .map(s => stripQuotes(s.trim()))
        .filter(s => s !== '')
      i++
    } else {
      result[key] = parseScalar(raw)
      i++
    }
  }

  return result
}

function parseScalar(s) {
  if (s === 'true')  return true
  if (s === 'false') return false
  if (s === 'null' || s === '~') return null
  const unquoted = stripQuotes(s)
  if (unquoted !== s) return unquoted   // was quoted — return as string
  const n = Number(s)
  if (s !== '' && !isNaN(n)) return n
  return s
}

function stripQuotes(s) {
  if (
    (s.startsWith('"') && s.endsWith('"')) ||
    (s.startsWith("'") && s.endsWith("'"))
  ) {
    return s.slice(1, -1)
  }
  return s
}
