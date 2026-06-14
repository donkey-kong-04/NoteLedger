<script lang="ts">
  import { CAT_COLORS, contrastText } from '../types';

  export let label: string;
  export let catType: string;
  export let selected = false;
  export let clickable = true;
  export let size: 'sm' | 'md' = 'sm';

  $: color = CAT_COLORS[catType]?.hex ?? '#888';
  $: textColor = selected ? contrastText(color) : color;
  $: bg = selected ? color : 'transparent';
</script>

<button
  class="badge"
  class:selected
  class:sm={size === 'sm'}
  class:md={size === 'md'}
  class:clickable
  style="--color: {color}; --bg: {bg}; --text: {textColor};"
  on:click
  disabled={!clickable}
>
  {label}
</button>

<style>
  .badge {
    display: inline-flex;
    align-items: center;
    border: 1.5px solid var(--color);
    border-radius: 999px;
    background: var(--bg);
    color: var(--text);
    font-weight: 500;
    cursor: default;
    transition: background 0.15s, color 0.15s, transform 0.1s;
    white-space: nowrap;
    font-family: inherit;
  }
  .badge.clickable { cursor: pointer; }
  .badge.clickable:hover { opacity: 0.85; transform: translateY(-1px); }
  .badge.clickable:active { transform: translateY(0); }
  .sm { font-size: 11px; padding: 2px 10px; }
  .md { font-size: 12px; padding: 4px 12px; }
</style>
