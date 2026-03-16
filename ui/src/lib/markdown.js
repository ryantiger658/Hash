/**
 * Markdown rendering for #ash.
 * Uses marked with syntax highlighting (highlight.js) and custom extensions.
 */
import { marked } from 'marked'
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

marked.use({
  extensions: [wikiLinkExtension],
  gfm: true,
  breaks: false,
  renderer: {
    // Fenced code blocks: syntax highlight with hljs
    code({ text, lang }) {
      const language = (lang ?? '').split(/\s+/)[0]
      if (language && hljs.getLanguage(language)) {
        const highlighted = hljs.highlight(text, { language }).value
        return `<pre><code class="hljs language-${language}">${highlighted}</code></pre>`
      }
      return `<pre><code class="hljs">${hljs.highlightAuto(text).value}</code></pre>`
    },
    // Inline code: add color chip for hex colors
    codespan({ text }) {
      if (HEX_RE.test(text)) {
        return `<code><span class="color-chip" style="background:${text}"></span>${text}</code>`
      }
      return false // use default rendering
    },
  },
})

/**
 * Render markdown to an HTML string.
 * Task-list checkboxes have `disabled` removed so they can be interacted with;
 * each gets a `data-cb` index for toggling back in the source.
 * @param {string} md
 * @returns {string}
 */
export function renderMarkdown(md) {
  const raw = marked.parse(md ?? '')
  // Remove `disabled=""` from task-list checkboxes and stamp a data-cb index
  let cbIndex = 0
  return raw.replace(/<input\b([^>]*?)\btype="checkbox"\b([^>]*)>/g, (match, pre, post) => {
    const attrs = (pre + post)
      .replace(/\s*disabled=""/g, '')
      .replace(/\s*disabled\b/g, '')
      .trim()
    return `<input type="checkbox"${attrs ? ' ' + attrs : ''} data-cb="${cbIndex++}">`
  })
}
