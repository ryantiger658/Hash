/**
 * Markdown rendering for #ash.
 * Uses marked with a wiki-link extension that emits clickable data-target anchors.
 */
import { marked } from 'marked'

// Wiki-link extension: [[Note Name]] or [[Note Name|Display Text]]
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

marked.use({
  extensions: [wikiLinkExtension],
  gfm: true,        // GitHub Flavored Markdown
  breaks: false,    // don't turn single newlines into <br>
})

/**
 * Render markdown to an HTML string.
 * @param {string} md
 * @returns {string}
 */
export function renderMarkdown(md) {
  return marked.parse(md ?? '')
}
