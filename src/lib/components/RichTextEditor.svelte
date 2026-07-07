<script lang="ts">
  import { onMount } from 'svelte';
  import { handleLinkClick } from '../types';
  import { sanitizeHtml } from '../sanitize';

  export let value: string = '';

  let editor: HTMLDivElement;
  let activeFormats = { bold: false, italic: false, ul: false, ol: false, inTable: false };

  const PICKER_MAX = 6;
  let showTablePicker = false;
  let pickRows = 0;
  let pickCols = 0;

  const FONT_SIZES = [
    { label: 'Small',  px: '12px' },
    { label: 'Normal', px: '14px' },
    { label: 'Large',  px: '18px' },
    { label: 'Huge',   px: '24px' },
  ];

  const COLORS = [
    '#111827', '#6b7280', '#ef4444', '#f59e0b',
    '#22c55e', '#3b82f6', '#8b5cf6', '#ec4899',
  ];

  onMount(() => {
    if (editor && value) editor.innerHTML = sanitizeHtml(value);
    document.execCommand('styleWithCSS', false, 'true');
  });

  function exec(cmd: string, val?: string) {
    editor.focus();
    document.execCommand(cmd, false, val ?? '');
    syncValue();
    updateActive();
  }

  function setSize(px: string) {
    const sel = window.getSelection();
    if (!sel || sel.rangeCount === 0) return;
    const range = sel.getRangeAt(0);
    if (range.collapsed) return;

    const span = document.createElement('span');
    span.style.fontSize = px;
    try {
      range.surroundContents(span);
    } catch {
      const frag = range.extractContents();
      span.appendChild(frag);
      range.insertNode(span);
    }
    sel.removeAllRanges();
    const newRange = document.createRange();
    newRange.selectNodeContents(span);
    sel.addRange(newRange);
    syncValue();
  }

  function setColor(color: string) {
    exec('foreColor', color);
  }

  function clearFormat() {
    exec('removeFormat');
  }

  function insertTable(rows: number, cols: number) {
    showTablePicker = false;
    const headerCells = '<th><br></th>'.repeat(cols);
    const bodyRow = `<tr>${'<td><br></td>'.repeat(cols)}</tr>`;
    const html =
      `<table><tbody><tr>${headerCells}</tr>${bodyRow.repeat(Math.max(rows - 1, 0))}</tbody></table><div><br></div>`;
    exec('insertHTML', html);
  }

  function currentCell(): HTMLTableCellElement | null {
    const cell = closestEl(window.getSelection()?.anchorNode)?.closest('td, th');
    return cell && editor?.contains(cell) ? (cell as HTMLTableCellElement) : null;
  }

  function newCell(header: boolean): HTMLTableCellElement {
    const c = document.createElement(header ? 'th' : 'td');
    c.appendChild(document.createElement('br'));
    return c as HTMLTableCellElement;
  }

  function addRow() {
    const cell = currentCell();
    const row = cell?.closest('tr');
    if (!cell || !row) return;
    const newRow = document.createElement('tr');
    for (let i = 0; i < row.children.length; i++) newRow.appendChild(newCell(false));
    row.after(newRow);
    placeCaret(newRow.children[0]);
    syncValue();
    updateActive();
  }

  function removeRow() {
    const cell = currentCell();
    const row = cell?.closest('tr');
    const table = cell?.closest('table');
    if (!cell || !row || !table) return;
    if (table.querySelectorAll('tr').length <= 1) {
      table.remove();
    } else {
      const neighbor = row.nextElementSibling ?? row.previousElementSibling;
      row.remove();
      const target = neighbor?.querySelector('td, th');
      if (target) placeCaret(target);
    }
    syncValue();
    updateActive();
  }

  function addColumn() {
    const cell = currentCell();
    const table = cell?.closest('table');
    if (!cell || !table) return;
    const idx = cell.cellIndex;
    table.querySelectorAll('tr').forEach(r => {
      const ref = r.children[idx];
      const c = newCell(ref?.tagName === 'TH');
      ref ? ref.after(c) : r.appendChild(c);
    });
    syncValue();
  }

  function removeColumn() {
    const cell = currentCell();
    const table = cell?.closest('table');
    if (!cell || !table) return;
    const idx = cell.cellIndex;
    const firstRow = table.querySelector('tr');
    if (!firstRow || firstRow.children.length <= 1) {
      table.remove();
    } else {
      table.querySelectorAll('tr').forEach(r => r.children[idx]?.remove());
      const target = table.querySelector('td, th');
      if (target) placeCaret(target);
    }
    syncValue();
    updateActive();
  }

  function closestEl(node: Node | null | undefined): Element | null {
    if (!node) return null;
    return node.nodeType === Node.ELEMENT_NODE ? (node as Element) : node.parentElement;
  }

  function placeCaret(cell: Element) {
    const sel = window.getSelection();
    if (!sel) return;
    const range = document.createRange();
    range.selectNodeContents(cell);
    range.collapse(true);
    sel.removeAllRanges();
    sel.addRange(range);
  }

  // Tab in a table moves between cells; Tab in the last cell appends a row.
  function handleTableTab(cell: Element, backwards: boolean): void {
    const table = cell.closest('table');
    if (!table) return;
    const cells = Array.from(table.querySelectorAll('th, td'));
    const idx = cells.indexOf(cell);
    if (backwards) {
      if (idx > 0) placeCaret(cells[idx - 1]);
      return;
    }
    if (idx < cells.length - 1) {
      placeCaret(cells[idx + 1]);
      return;
    }
    const lastRow = cell.closest('tr');
    if (!lastRow) return;
    const newRow = document.createElement('tr');
    for (let i = 0; i < lastRow.children.length; i++) {
      const td = document.createElement('td');
      td.appendChild(document.createElement('br'));
      newRow.appendChild(td);
    }
    lastRow.after(newRow);
    placeCaret(newRow.children[0]);
    syncValue();
  }

  function syncValue() {
    value = sanitizeHtml(editor.innerHTML);
  }

  function updateActive() {
    activeFormats = {
      bold: document.queryCommandState('bold'),
      italic: document.queryCommandState('italic'),
      ul: document.queryCommandState('insertUnorderedList'),
      ol: document.queryCommandState('insertOrderedList'),
      inTable: !!currentCell(),
    };
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Tab') {
      e.preventDefault();
      const sel = window.getSelection();
      const node = sel?.anchorNode;
      const cell = closestEl(node)?.closest('td, th');
      if (cell) {
        handleTableTab(cell, e.shiftKey);
        return;
      }
      const inList = !!(node && (node as Element).closest?.('li, ul, ol') ||
        (node?.parentElement?.closest('li, ul, ol')));
      if (inList) {
        document.execCommand(e.shiftKey ? 'outdent' : 'indent');
      } else if (!e.shiftKey) {
        document.execCommand('insertText', false, '    ');
      }
      syncValue();
    }
  }

  function onPaste(e: ClipboardEvent) {
    const text = e.clipboardData?.getData('text/plain') ?? '';
    const html = e.clipboardData?.getData('text/html') ?? '';
    if (html) {
      e.preventDefault();
      document.execCommand('insertHTML', false, sanitizeHtml(html));
      syncValue();
      return;
    }
    if (/^https?:\/\/\S+$/.test(text.trim())) {
      e.preventDefault();
      // Build the anchor via DOM so the URL is attribute-escaped when serialized.
      const a = document.createElement('a');
      a.href = text.trim();
      a.textContent = text.trim();
      document.execCommand('insertHTML', false, a.outerHTML);
      syncValue();
    }
  }
</script>

<div class="rte">
  <div class="toolbar">
    <button
      class="tb-btn" class:active={activeFormats.bold}
      on:mousedown|preventDefault={() => exec('bold')}
      title="Bold (⌘B)"
    ><b>B</b></button>

    <button
      class="tb-btn" class:active={activeFormats.italic}
      on:mousedown|preventDefault={() => exec('italic')}
      title="Italic (⌘I)"
    ><i>I</i></button>

    <button
      class="tb-btn" class:active={activeFormats.ul}
      on:mousedown|preventDefault={() => exec('insertUnorderedList')}
      title="Bullet list"
    >
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
        <circle cx="2" cy="4" r="1" fill="currentColor" stroke="none"/>
        <line x1="5" y1="4" x2="13" y2="4"/>
        <circle cx="2" cy="8" r="1" fill="currentColor" stroke="none"/>
        <line x1="5" y1="8" x2="13" y2="8"/>
        <circle cx="2" cy="12" r="1" fill="currentColor" stroke="none"/>
        <line x1="5" y1="12" x2="13" y2="12"/>
      </svg>
    </button>

    <button
      class="tb-btn" class:active={activeFormats.ol}
      on:mousedown|preventDefault={() => exec('insertOrderedList')}
      title="Numbered list"
    >
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round">
        <text x="0" y="5" font-size="5" font-weight="700" fill="currentColor" stroke="none" font-family="sans-serif">1.</text>
        <line x1="5" y1="4" x2="13" y2="4"/>
        <text x="0" y="9" font-size="5" font-weight="700" fill="currentColor" stroke="none" font-family="sans-serif">2.</text>
        <line x1="5" y1="8" x2="13" y2="8"/>
        <text x="0" y="13" font-size="5" font-weight="700" fill="currentColor" stroke="none" font-family="sans-serif">3.</text>
        <line x1="5" y1="12" x2="13" y2="12"/>
      </svg>
    </button>

    <div class="tb-divider"></div>

    <select class="tb-select" on:change={e => { setSize((e.target as HTMLSelectElement).value); (e.target as HTMLSelectElement).value = ''; }} title="Font size">
      <option value="" disabled selected>Size</option>
      {#each FONT_SIZES as s}
        <option value={s.px}>{s.label}</option>
      {/each}
    </select>

    <div class="tb-table-wrap">
      <button
        class="tb-btn" class:active={showTablePicker}
        on:mousedown|preventDefault={() => { showTablePicker = !showTablePicker; pickRows = 0; pickCols = 0; }}
        title="Insert table"
      >
        <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round">
          <rect x="1" y="1" width="12" height="12" rx="1"/>
          <line x1="1" y1="5" x2="13" y2="5"/>
          <line x1="1" y1="9" x2="13" y2="9"/>
          <line x1="5.5" y1="1" x2="5.5" y2="13"/>
          <line x1="9.5" y1="1" x2="9.5" y2="13"/>
        </svg>
      </button>
      {#if showTablePicker}
        <div class="table-picker" role="presentation" on:mousedown|preventDefault>
          <div class="picker-grid">
            {#each Array(PICKER_MAX) as _, r}
              {#each Array(PICKER_MAX) as _, c}
                <!-- svelte-ignore a11y-no-static-element-interactions -->
                <div
                  class="pick-cell"
                  class:lit={r < pickRows && c < pickCols}
                  on:mouseenter={() => { pickRows = r + 1; pickCols = c + 1; }}
                  on:mousedown|preventDefault={() => insertTable(r + 1, c + 1)}
                ></div>
              {/each}
            {/each}
          </div>
          <div class="picker-label">{pickRows > 0 ? `${pickCols} × ${pickRows}` : 'Pick size'}</div>
        </div>
      {/if}
    </div>

    <button
      class="tb-btn"
      on:mousedown|preventDefault={clearFormat}
      title="Clear formatting"
    >
      <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round">
        <path d="M3 2h8M7 2 5.5 10"/>
        <line x1="3.5" y1="12" x2="7.5" y2="12"/>
        <line x1="9" y1="8.5" x2="12.5" y2="12"/>
        <line x1="12.5" y1="8.5" x2="9" y2="12"/>
      </svg>
    </button>

    <div class="tb-divider"></div>

    {#each COLORS as color}
      <button
        class="color-swatch"
        style="background:{color}"
        on:mousedown|preventDefault={() => setColor(color)}
        title={color}
      ></button>
    {/each}

    {#if activeFormats.inTable}
      <div class="tb-divider"></div>
      <button class="tb-txt-btn" on:mousedown|preventDefault={addRow} title="Add row below">+ Row</button>
      <button class="tb-txt-btn" on:mousedown|preventDefault={removeRow} title="Remove current row">− Row</button>
      <button class="tb-txt-btn" on:mousedown|preventDefault={addColumn} title="Add column after">+ Col</button>
      <button class="tb-txt-btn" on:mousedown|preventDefault={removeColumn} title="Remove current column">− Col</button>
    {/if}
  </div>

  <div
    class="editor"
    bind:this={editor}
    contenteditable="true"
    role="textbox"
    aria-multiline="true"
    aria-label="Description"
    on:focus={() => showTablePicker = false}
    on:input={syncValue}
    on:keyup={updateActive}
    on:mouseup={updateActive}
    on:keydown={onKeydown}
    on:click={handleLinkClick}
    on:paste={onPaste}
  ></div>
</div>

<style>
  .rte {
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
    transition: border-color 0.15s;
    background: var(--surface-2);
  }
  .rte:focus-within { border-color: var(--accent); }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 3px;
    padding: 5px 8px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    flex-wrap: wrap;
  }

  .tb-btn {
    width: 28px; height: 26px;
    background: none; border: none;
    border-radius: 5px;
    cursor: pointer;
    font-size: 13px;
    color: var(--text-muted);
    display: flex; align-items: center; justify-content: center;
    transition: background 0.1s, color 0.1s;
    font-family: inherit;
  }
  .tb-btn:hover { background: var(--surface-2); color: var(--text); }
  .tb-btn.active { background: var(--surface-3); color: var(--accent); }

  .tb-select {
    height: 26px;
    padding: 0 6px;
    font-size: 12px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 5px;
    color: var(--text);
    cursor: pointer;
    font-family: inherit;
    outline: none;
    width: auto;
  }
  .tb-select:focus { border-color: var(--accent); }

  .tb-table-wrap { position: relative; }

  .table-picker {
    position: absolute;
    top: 30px; left: 0;
    z-index: 10;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 8px;
    box-shadow: 0 4px 16px rgba(0,0,0,0.15);
  }

  .picker-grid {
    display: grid;
    grid-template-columns: repeat(6, 14px);
    gap: 3px;
  }

  .pick-cell {
    width: 14px; height: 14px;
    border: 1px solid var(--border);
    border-radius: 3px;
    background: var(--surface-2);
    cursor: pointer;
  }
  .pick-cell.lit {
    background: var(--accent);
    border-color: var(--accent);
    opacity: 0.75;
  }

  .picker-label {
    margin-top: 6px;
    font-size: 11px;
    color: var(--text-muted);
    text-align: center;
  }

  .tb-txt-btn {
    height: 26px;
    padding: 0 7px;
    background: none;
    border: none;
    border-radius: 5px;
    cursor: pointer;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    font-family: inherit;
    white-space: nowrap;
    transition: background 0.1s, color 0.1s;
  }
  .tb-txt-btn:hover { background: var(--surface-2); color: var(--text); }

  .tb-divider {
    width: 1px; height: 18px;
    background: var(--border);
    margin: 0 3px;
    flex-shrink: 0;
  }

  .color-swatch {
    width: 18px; height: 18px;
    border-radius: 50%;
    border: 1.5px solid transparent;
    cursor: pointer;
    flex-shrink: 0;
    transition: transform 0.1s, border-color 0.1s;
    padding: 0;
  }
  .color-swatch:hover {
    transform: scale(1.25);
    border-color: var(--border);
  }

  .editor {
    min-height: 110px;
    padding: 10px 12px;
    color: var(--text);
    font-size: 14px;
    font-family: inherit;
    line-height: 1.6;
    outline: none;
    overflow-y: auto;
  }
  .editor:empty::before {
    content: attr(placeholder);
    color: var(--text-muted);
    pointer-events: none;
  }
  .editor :global(ul),
  .editor :global(ol) {
    margin: 4px 0;
    padding-left: 20px;
  }
  .editor :global(li) {
    margin: 2px 0;
  }
  .editor :global(table) {
    border-collapse: collapse;
    margin: 6px 0;
    width: 100%;
  }
  .editor :global(th),
  .editor :global(td) {
    border: 1px solid var(--border);
    padding: 4px 8px;
    text-align: left;
    vertical-align: top;
  }
  .editor :global(th) {
    background: var(--surface);
    font-weight: 600;
  }
</style>
