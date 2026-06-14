<script lang="ts">
  import { onMount } from 'svelte';

  export let value: string = '';

  let editor: HTMLDivElement;
  let activeFormats = { bold: false, italic: false };

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
    if (editor && value) editor.innerHTML = value;
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

  function syncValue() {
    value = editor.innerHTML;
  }

  function updateActive() {
    activeFormats = {
      bold: document.queryCommandState('bold'),
      italic: document.queryCommandState('italic'),
    };
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Tab') {
      e.preventDefault();
      document.execCommand('insertText', false, '    ');
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

    <div class="tb-divider"></div>

    <select class="tb-select" on:change={e => { setSize((e.target as HTMLSelectElement).value); (e.target as HTMLSelectElement).value = ''; }} title="Font size">
      <option value="" disabled selected>Size</option>
      {#each FONT_SIZES as s}
        <option value={s.px}>{s.label}</option>
      {/each}
    </select>

    <div class="tb-divider"></div>

    {#each COLORS as color}
      <button
        class="color-swatch"
        style="background:{color}"
        on:mousedown|preventDefault={() => setColor(color)}
        title={color}
      ></button>
    {/each}
  </div>

  <div
    class="editor"
    bind:this={editor}
    contenteditable="true"
    role="textbox"
    aria-multiline="true"
    aria-label="Description"
    on:input={syncValue}
    on:keyup={updateActive}
    on:mouseup={updateActive}
    on:keydown={onKeydown}
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
</style>
