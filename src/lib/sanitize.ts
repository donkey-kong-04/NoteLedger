import DOMPurify from 'dompurify';

// Descriptions are stored as raw editor HTML, so every path that turns that
// HTML back into DOM ({@html ...} renders, editor.innerHTML loads) must go
// through this. The allowlist covers exactly what RichTextEditor produces:
// formatting tags, lists, tables, styled spans and pasted links. Event
// handlers, scripts and javascript: URLs are stripped.
const ALLOWED_TAGS = [
  'a', 'b', 'i', 'u', 's', 'em', 'strong',
  'p', 'div', 'span', 'br',
  'ul', 'ol', 'li',
  'table', 'thead', 'tbody', 'tr', 'th', 'td',
  'font',
];

const ALLOWED_ATTR = ['href', 'style', 'color', 'colspan', 'rowspan'];

export function sanitizeHtml(html: string): string {
  return DOMPurify.sanitize(html, { ALLOWED_TAGS, ALLOWED_ATTR });
}
