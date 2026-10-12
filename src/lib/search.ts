import type { Log } from './types';
import { sanitizeHtml } from './sanitize';

// Log search: whitespace-separated tokens, each matched case-insensitively as
// a substring. AND across tokens, but each token may sit in any field (title,
// open points, closed points) — "abc ghi" matches "abc def ghi" and
// "abcdefghi" alike.

export function parseSearchTokens(query: string): string[] {
  return [...new Set(query.toLowerCase().split(/\s+/).filter(Boolean))];
}

function tokenRegex(token: string, flags: string): RegExp {
  return new RegExp(token.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), flags);
}

// Fields are tested one by one so a token never matches across two fields.
export function fieldsMatchAllTokens(fields: string[], tokens: string[]): boolean {
  return tokens.every(t => {
    const re = tokenRegex(t, 'iu');
    return fields.some(f => re.test(f));
  });
}

// Merged [start, end) ranges of every token occurrence — overlapping tokens
// ("abc" + "bcd" in "abcd") collapse into one range.
export function matchRanges(text: string, tokens: string[]): [number, number][] {
  const ranges: [number, number][] = [];
  for (const t of tokens) {
    for (const m of text.matchAll(tokenRegex(t, 'giu'))) {
      ranges.push([m.index, m.index + m[0].length]);
    }
  }
  ranges.sort((a, b) => a[0] - b[0]);
  const merged: [number, number][] = [];
  for (const [s, e] of ranges) {
    const last = merged[merged.length - 1];
    if (last && s <= last[1]) last[1] = Math.max(last[1], e);
    else merged.push([s, e]);
  }
  return merged;
}

// Plain-text highlighting (log titles), rendered as text + <mark> in Svelte.
export function highlightSegments(text: string, tokens: string[]): { text: string; hit: boolean }[] {
  const ranges = tokens.length ? matchRanges(text, tokens) : [];
  if (!ranges.length) return [{ text, hit: false }];
  const out: { text: string; hit: boolean }[] = [];
  let pos = 0;
  for (const [s, e] of ranges) {
    if (s > pos) out.push({ text: text.slice(pos, s), hit: false });
    out.push({ text: text.slice(s, e), hit: true });
    pos = e;
  }
  if (pos < text.length) out.push({ text: text.slice(pos), hit: false });
  return out;
}

// ── Rich text (needs the DOM) ───────────────────────────────────────────────

function parseFragment(html: string): DocumentFragment {
  const tpl = document.createElement('template');
  tpl.innerHTML = html;
  return tpl.content;
}

function textNodes(root: Node): Text[] {
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  const nodes: Text[] = [];
  for (let n = walker.nextNode(); n; n = walker.nextNode()) nodes.push(n as Text);
  return nodes;
}

// Text of a rich-text field as displayed: sanitized, then the concatenation of
// its text nodes — exactly what highlightHtml() walks, so whatever matches
// gets highlighted. Cached: filtering re-runs on every keystroke.
const textCache = new Map<string, string>();
export function htmlToText(html: string | null | undefined): string {
  if (!html) return '';
  let text = textCache.get(html);
  if (text === undefined) {
    text = parseFragment(sanitizeHtml(html)).textContent ?? '';
    if (textCache.size > 5000) textCache.clear();
    textCache.set(html, text);
  }
  return text;
}

export function logMatchesSearch(log: Log, tokens: string[]): boolean {
  if (tokens.length === 0) return true;
  return fieldsMatchAllTokens(
    [log.title, htmlToText(log.description), htmlToText(log.closed_description)],
    tokens,
  );
}

// Wraps every token occurrence in already-sanitized HTML with
// <mark class="search-hit">. Ranges are found on the concatenated text, so a
// match spanning formatting (half-bold word) is still highlighted, one <mark>
// per text-node piece. Built with DOM APIs, so no new markup is injected.
export function highlightHtml(html: string, tokens: string[]): string {
  if (!html || tokens.length === 0) return html;
  const root = parseFragment(html);
  const nodes = textNodes(root);
  const ranges = matchRanges(nodes.map(n => n.data).join(''), tokens);
  if (!ranges.length) return html;

  let offset = 0;
  for (const node of nodes) {
    const start = offset;
    const end = offset + node.data.length;
    offset = end;
    const local = ranges
      .filter(([s, e]) => s < end && e > start)
      .map(([s, e]) => [Math.max(s, start) - start, Math.min(e, end) - start]);
    if (!local.length) continue;

    const frag = document.createDocumentFragment();
    let pos = 0;
    for (const [s, e] of local) {
      if (s > pos) frag.append(node.data.slice(pos, s));
      const mark = document.createElement('mark');
      mark.className = 'search-hit';
      mark.textContent = node.data.slice(s, e);
      frag.append(mark);
      pos = e;
    }
    if (pos < node.data.length) frag.append(node.data.slice(pos));
    node.replaceWith(frag);
  }

  const out = document.createElement('div');
  out.append(root);
  return out.innerHTML;
}
