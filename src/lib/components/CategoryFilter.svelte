<script lang="ts">
  import Badge from './Badge.svelte';
  import { createPicklistValue, deletePicklistValue, updatePicklistValue } from '../store';
  import type { PicklistValue } from '../types';
  import { createEventDispatcher } from 'svelte';
  import { CAT_COLORS } from '../types';

  const dispatch = createEventDispatcher();

  $: color = CAT_COLORS[catType]?.hex ?? '#888';

  export let catType: string;
  export let label: string;
  export let values: PicklistValue[];
  export let selected: number[];
  export let layout: 'vertical' | 'horizontal';

  let editingLabel = false;
  let labelDraft = label;
  let searchText = '';
  let showSearch = false;
  let inputEl: HTMLInputElement;
  let isClicking = false;

  function filterValues(vals: PicklistValue[], q: string) {
    const query = q.trim().toLowerCase();
    return query ? vals.filter(v => v.label.toLowerCase().includes(query)) : vals;
  }

  function toggleValue(id: number) {
    if (selected.includes(id)) {
      selected = selected.filter(x => x !== id);
    } else {
      selected = [...selected, id];
    }
    searchText = '';
  }

  let lastError = '';

  async function removeValue(id: number) {
    try {
      await deletePicklistValue(id);
      selected = selected.filter(x => x !== id);
      lastError = '';
    } catch (e: any) {
      lastError = String(e);
    }
  }

  let editingId: number | null = null;
  let editDraft = '';

  function startEdit(val: PicklistValue) {
    editingId = val.id;
    editDraft = val.label;
  }

  async function commitEdit(id: number) {
    const trimmed = editDraft.trim();
    editingId = null;
    if (!trimmed) return;
    try {
      await updatePicklistValue(id, trimmed);
      lastError = '';
    } catch (e: any) {
      lastError = String(e);
    }
  }

  function cancelEdit() { editingId = null; }

  async function handleEnter() {
    const trimmed = searchText.trim();
    if (!trimmed) { closeSearch(); return; }
    // If exact match exists, toggle it instead of creating
    const exact = values.find(v => v.label.toLowerCase() === trimmed.toLowerCase());
    if (exact) {
      toggleValue(exact.id);
      closeSearch();
      return;
    }
    // Otherwise create a new value and auto-select it
    const type = catType;
    searchText = '';
    showSearch = false;
    try {
      const newVal = await createPicklistValue(type, trimmed);
      toggleValue(newVal.id);
      lastError = '';
    } catch (e) {
      lastError = `Error [${type}]: ${e}`;
    }
  }

  function closeSearch() {
    if (isClicking) return;
    showSearch = false;
    searchText = '';
  }

  function openSearch() {
    showSearch = true;
  }

  function commitLabel() {
    editingLabel = false;
    label = labelDraft;
    dispatch('labelChange');
  }
</script>

<div class="cat-filter" class:horizontal={layout === 'horizontal'} class:vertical={layout === 'vertical'}>
  <div class="cat-header">
    {#if editingLabel}
      <input
        class="label-input"
        bind:value={labelDraft}
        on:blur={commitLabel}
        on:keydown={e => e.key === 'Enter' && commitLabel()}
        autofocus
      />
    {:else}
      <!-- svelte-ignore a11y-click-events-have-key-events -->
      <!-- svelte-ignore a11y-no-static-element-interactions -->
      <span class="cat-label" style="color: {color};" on:click={() => { editingLabel = true; labelDraft = label; }}>{label}</span>
    {/if}
  </div>

  {#if lastError}
    <span class="err">{lastError}</span>
  {/if}

  <!-- svelte-ignore a11y-no-static-element-interactions -->
  <div class="badges" class:wrap={layout === 'horizontal'}
    on:mousedown={() => { isClicking = true; }}
    on:mouseup={() => { isClicking = false; }}
  >
    <!-- Search/add input always at the beginning -->
    {#if showSearch}
      <input
        class="search-input"
        style="border-color: {color}; outline-color: {color}"
        bind:value={searchText}
        bind:this={inputEl}
        placeholder="Search or add…"
        on:blur={closeSearch}
        on:keydown={e => {
          if (e.key === 'Enter') { e.preventDefault(); handleEnter(); }
          if (e.key === 'Escape') { isClicking = false; closeSearch(); }
        }}
        autofocus
      />
    {:else}
      <button class="add-btn" on:click={openSearch}>Add or search</button>
    {/if}

    {#each filterValues(values, searchText) as val (val.id)}
      <div class="badge-row">
        {#if editingId === val.id}
          <input
            class="edit-val-input"
            bind:value={editDraft}
            on:blur={() => commitEdit(val.id)}
            on:keydown={e => { if (e.key === 'Enter') commitEdit(val.id); if (e.key === 'Escape') cancelEdit(); }}
            autofocus
          />
        {:else}
          <Badge
            label={val.label}
            catType={catType}
            selected={selected.includes(val.id)}
            on:click={() => toggleValue(val.id)}
          />
          {#if selected.includes(val.id)}
            <button class="action-btn" on:click={() => startEdit(val)} title="Edit value">✎</button>
            <button class="action-btn del" on:click={() => removeValue(val.id)} title="Delete value">×</button>
          {/if}
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .cat-filter { display: flex; gap: 6px; }
  .cat-filter.vertical { flex-direction: column; }
  .cat-filter.horizontal { flex-direction: column; }

  .cat-header { margin-bottom: 2px; }

  .cat-label {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    cursor: pointer;
    user-select: none;
    opacity: 0.85;
    transition: opacity 0.15s;
  }
  .cat-label:hover { opacity: 1; }

  .label-input {
    font-size: 11px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    border: none;
    background: transparent;
    color: var(--text);
    outline: 1px solid var(--accent);
    border-radius: 3px;
    padding: 0 3px;
    width: 90px;
  }

  .badges {
    display: flex;
    gap: 6px;
    flex-direction: row;
    flex-wrap: wrap;
    align-items: center;
  }

  /* In the drawer, long value lists scroll instead of growing the panel */
  .cat-filter.vertical .badges {
    max-height: 180px;
    overflow-y: auto;
    align-content: flex-start;
    padding-right: 4px;
  }

  .badge-row {
    display: flex; align-items: center; gap: 3px;
  }

  .action-btn {
    display: inline-flex; align-items: center;
    background: none; border: none; cursor: pointer;
    color: var(--text-muted); font-size: 13px; line-height: 1;
    padding: 0 2px; flex-shrink: 0;
  }
  .action-btn:hover { color: var(--text); }
  .action-btn.del:hover { color: #ef4444; }

  .edit-val-input {
    font-size: 11px;
    border: 1.5px solid var(--accent);
    border-radius: 999px;
    padding: 2px 10px;
    background: var(--surface);
    color: var(--text);
    outline: none;
    width: 100px;
    font-family: inherit;
  }

  .add-btn {
    font-size: 11px;
    color: var(--text-muted);
    background: none;
    border: 1.5px dashed var(--border);
    border-radius: 999px;
    padding: 2px 10px;
    cursor: pointer;
    transition: color 0.15s, border-color 0.15s;
    font-family: inherit;
    white-space: nowrap;
  }
  .add-btn:hover { color: var(--text); border-color: var(--text-muted); }

  .err { font-size: 10px; color: #ef4444; word-break: break-all; }

  .search-input {
    font-size: 11px;
    border: 1.5px solid var(--accent);
    border-radius: 999px;
    padding: 2px 10px;
    background: var(--surface);
    color: var(--text);
    outline: none;
    width: 110px;
    font-family: inherit;
  }
</style>
