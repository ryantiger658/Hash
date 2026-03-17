/**
 * Markdown rendering for #ash.
 * Uses marked with syntax highlighting (highlight.js) and custom extensions.
 */
import { Marked } from 'marked'
import hljs from 'highlight.js/lib/core'

// ── Register common languages ─────────────────────────────────────────────────
import javascript from 'highlight.js/lib/languages/javascript'
import typescript from 'highlight.js/lib/languages/typescript'
import python from 'highlight.js/lib/languages/python'
import rust from 'highlight.js/lib/languages/rust'
import bash from 'highlight.js/lib/languages/bash'
import yaml from 'highlight.js/lib/languages/yaml'
import toml from 'highlight.js/lib/languages/ini'   // TOML uses ini highlighter
import json from 'highlight.js/lib/languages/json'
import xml from 'highlight.js/lib/languages/xml'    // HTML/XML
import css from 'highlight.js/lib/languages/css'
import markdown from 'highlight.js/lib/languages/markdown'
import go from 'highlight.js/lib/languages/go'
import sql from 'highlight.js/lib/languages/sql'

hljs.registerLanguage('javascript', javascript)
hljs.registerLanguage('js', javascript)
hljs.registerLanguage('typescript', typescript)
hljs.registerLanguage('ts', typescript)
hljs.registerLanguage('python', python)
hljs.registerLanguage('py', python)
hljs.registerLanguage('rust', rust)
hljs.registerLanguage('rs', rust)
hljs.registerLanguage('bash', bash)
hljs.registerLanguage('sh', bash)
hljs.registerLanguage('shell', bash)
hljs.registerLanguage('yaml', yaml)
hljs.registerLanguage('yml', yaml)
hljs.registerLanguage('toml', toml)
hljs.registerLanguage('json', json)
hljs.registerLanguage('html', xml)
hljs.registerLanguage('xml', xml)
hljs.registerLanguage('css', css)
hljs.registerLanguage('markdown', markdown)
hljs.registerLanguage('md', markdown)
hljs.registerLanguage('go', go)
hljs.registerLanguage('sql', sql)

// ── Wiki-link extension ───────────────────────────────────────────────────────
const wikiLinkExtension = {
  name: 'wikiLink',
  level: 'inline',
  start(src) {
    return src.indexOf('[[')
  },
  tokenizer(src) {
    const match = /^\[\[([^\]]+)\]\]/.exec(src)
    if (match) {
      const parts = match[1].split('|')
      return {
        type: 'wikiLink',
        raw: match[0],
        target: parts[0].trim(),
        label: (parts[1] ?? parts[0]).trim(),
      }
    }
  },
  renderer(token) {
    return `<a class="wiki-link" data-target="${encodeURIComponent(token.target)}" href="#">${token.label}</a>`
  },
}

// ── Hex color swatch in inline code ──────────────────────────────────────────
const HEX_RE = /^#[0-9a-fA-F]{3,8}$/

// ── Shared renderer options (code blocks, inline code) ───────────────────────
const baseRenderer = {
  // Fenced code blocks: syntax highlight with hljs
  code(text, lang) {
    const safeText = text ?? ''
    const language = (lang ?? '').split(/\s+/)[0]
    if (language && hljs.getLanguage(language)) {
      try {
        const highlighted = hljs.highlight(safeText, { language }).value
        return `<pre><code class="hljs language-${language}">${highlighted}</code></pre>`
      } catch (_) {}
    }
    const escaped = safeText
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
    return `<pre><code class="hljs">${escaped}</code></pre>`
  },
  // Inline code: add color chip for hex colors
  codespan(text) {
    if (HEX_RE.test(text)) {
      return `<code><span class="color-chip" style="background:${text}"></span>${text}</code>`
    }
    return `<code>${text}</code>`
  },
}

/**
 * Resolve a (possibly relative) image href against the note's directory to a
 * vault-relative path, then build a /api/vault-asset URL with the session token.
 *
 * @param {string} href    - Original href from the markdown source
 * @param {string} noteDir - Directory of the current note (e.g. "journal/2026/Mar")
 * @param {string} token   - Session token for vault-asset auth
 * @returns {string}       - URL to pass to <img src>
 */
function resolveAssetUrl(href, noteDir, token) {
  if (!href) return href
  // External URLs pass through untouched
  if (/^https?:\/\//.test(href) || href.startsWith('data:')) return href
  // Strip leading slash — treat as vault-root-relative
  const stripped = href.startsWith('/') ? href.slice(1) : href
  // Resolve relative to note directory
  let vaultPath
  if (noteDir && !href.startsWith('/')) {
    // Combine noteDir + href, then normalise away any ../
    const parts = (noteDir + '/' + stripped).split('/')
    const resolved = []
    for (const p of parts) {
      if (p === '..') resolved.pop()
      else if (p && p !== '.') resolved.push(p)
    }
    vaultPath = resolved.join('/')
  } else {
    vaultPath = stripped
  }
  return `/api/vault-asset/${vaultPath}${token ? `?token=${encodeURIComponent(token)}` : ''}`
}

/**
 * Render markdown to an HTML string.
 * Task-list checkboxes have `disabled` removed so they can be interacted with;
 * each gets a `data-cb` index for toggling back in the source.
 *
 * @param {string} md       - Markdown source
 * @param {string} notePath - Vault-relative path of the current note (for image resolution)
 * @param {string} token    - Session token for vault-asset image URLs
 * @returns {string}
 */
export function renderMarkdown(md, notePath = '', token = '') {
  const noteDir = notePath.includes('/')
    ? notePath.slice(0, notePath.lastIndexOf('/'))
    : ''

  const parser = new Marked({
    gfm: true,
    breaks: false,
    extensions: [wikiLinkExtension],
    renderer: {
      ...baseRenderer,
      image(href, title, text) {
        const src = resolveAssetUrl(href, noteDir, token)
        const titleAttr = title ? ` title="${title}"` : ''
        return `<img src="${src}" alt="${text ?? ''}"${titleAttr} loading="lazy">`
      },
    },
  })

  const raw = parser.parse(md ?? '')
  // Remove `disabled=""` from task-list checkboxes and stamp a data-cb index
  let cbIndex = 0
  return raw.replace(/<input\b([^>]*?)\btype="checkbox"([^>]*)>/g, (match, pre, post) => {
    const attrs = (pre + post)
      .replace(/\s*disabled=""/g, '')
      .replace(/\s*disabled\b/g, '')
      .trim()
    return `<input type="checkbox"${attrs ? ' ' + attrs : ''} data-cb="${cbIndex++}">`
  })
}
