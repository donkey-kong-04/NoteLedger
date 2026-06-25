<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Badge from './Badge.svelte';
  import RichTextEditor from './RichTextEditor.svelte';
  import type { Log, PicklistValue, Project } from '../types';
  import { CAT_COLORS } from '../types';
  import { createLog, updateLog, deleteLog, createPicklistValue, picklists } from '../store';

  export let log: Log | null = null;
  export let logTypes: PicklistValue[];
  export let cat1Vals: PicklistValue[];
  export let cat2Vals: PicklistValue[];
  export let cat3Vals: PicklistValue[];
  export let cat4Vals: PicklistValue[];
  export let cat1Label: string = 'Category 1';
  export let cat2Label: string = 'Category 2';
  export let cat3Label: string = 'Category 3';
  export let cat4Label: string = 'Category 4';
  export let allProjects: Project[] = [];

  const dispatch = createEventDispatcher();

  const isNew = !log?.id;

  let draft: Log = {
    id: 0,
    type_id: logTypes[0]?.id ?? 0,
    title: '',
    description: '',
    start_date: '',
    due_date: null,
    is_closed: false,
    closed_date: null,
    project_id: allProjects[0]?.id ?? 0,
    category1_ids: [],
    category2_ids: [],
    category3_ids: [],
    category4_ids: [],
    ...(log ?? {}),
    category1_ids: log?.category1_ids ? [...log.category1_ids] : [],
    category2_ids: log?.category2_ids ? [...log.category2_ids] : [],
    category3_ids: log?.category3_ids ? [...log.category3_ids] : [],
    category4_ids: log?.category4_ids ? [...log.category4_ids] : [],
  };

  let dueDateStr = draft.due_date ?? '';
  let confirmDelete = false;

  function toggleCat(slot: 1|2|3|4, id: number) {
    const key = `category${slot}_ids` as const;
    const cur = draft[key];
    draft = { ...draft, [key]: cur.includes(id) ? cur.filter(x => x !== id) : [...cur, id] };
    addDraft[slot] = '';
  }

  async function save() {
    if (!draft.title.trim() || !draft.project_id) return;
    await pendingAdd;
    draft.due_date = dueDateStr || null;
if (isNew) {
      await createLog(draft);
    } else {
      await updateLog(draft);
    }
    dispatch('close');
  }

  async function remove() {
    if (!confirmDelete) { confirmDelete = true; return; }
    await deleteLog(draft.id);
    dispatch('close');
  }

  function close() { dispatch('close'); }

  const byLabel = (a: PicklistValue, b: PicklistValue) => a.label.localeCompare(b.label);
  $: liveCat1 = $picklists.filter(v => v.picklist_type === 'category_1').sort(byLabel);
  $: liveCat2 = $picklists.filter(v => v.picklist_type === 'category_2').sort(byLabel);
  $: liveCat3 = $picklists.filter(v => v.picklist_type === 'category_3').sort(byLabel);
  $: liveCat4 = $picklists.filter(v => v.picklist_type === 'category_4').sort(byLabel);

  $: cats = [
    { slot: 1 as const, label: cat1Label, vals: liveCat1 },
    { slot: 2 as const, label: cat2Label, vals: liveCat2 },
    { slot: 3 as const, label: cat3Label, vals: liveCat3 },
    { slot: 4 as const, label: cat4Label, vals: liveCat4 },
  ];

  let showAddInput: Record<number, boolean> = { 1: false, 2: false, 3: false, 4: false };
  let addDraft: Record<number, string> = { 1: '', 2: '', 3: '', 4: '' };
  let pendingAdd: Promise<void> = Promise.resolve();
  let isClicking: Record<number, boolean> = { 1: false, 2: false, 3: false, 4: false };

  function filteredVals(vals: typeof cats[0]['vals'], q: string) {
    const query = q.trim().toLowerCase();
    return query ? vals.filter(v => v.label.toLowerCase().includes(query)) : vals;
  }

  async function addAndSelect(slot: 1|2|3|4, vals: typeof cats[0]['vals']) {
    const trimmed = addDraft[slot].trim();
    if (!trimmed) { addDraft[slot] = ''; showAddInput[slot] = false; return; }
    // Exact match → toggle existing value
    const exact = vals.find(v => v.label.toLowerCase() === trimmed.toLowerCase());
    if (exact) { toggleCat(slot, exact.id); addDraft[slot] = ''; showAddInput[slot] = false; return; }
    // No match → create new and select
    addDraft[slot] = '';
    showAddInput[slot] = false;
    const p = createPicklistValue(`category_${slot}`, trimmed).then(newVal => {
      toggleCat(slot, newVal.id);
    });
    pendingAdd = p;
    await p;
  }
</script>

<div class="backdrop" on:click={close} on:keydown={e => e.key === 'Escape' && close()} role="presentation"></div>

<div class="panel" role="dialog" aria-modal="true">
  <div class="panel-header">
    <h2>{isNew ? 'New Log' : 'Edit Log'}</h2>
    <button class="icon-btn" on:click={close} aria-label="Close">✕</button>
  </div>

  <div class="panel-body">
    <div class="field">
      <label>Title <span class="req">*</span></label>
      <input bind:value={draft.title} placeholder="What happened?" />
    </div>

    <div class="field">
      <label>Log Type <span class="req">*</span></label>
      <select bind:value={draft.type_id}>
        {#each logTypes as lt}
          <option value={lt.id}>{lt.label}</option>
        {/each}
      </select>
    </div>

    <div class="field">
      <label>Description</label>
      <RichTextEditor bind:value={draft.description} />
    </div>

    <div class="field-row">
      <div class="field">
        <label>Due Date</label>
        <input type="date" bind:value={dueDateStr} />
      </div>
      <div class="field">
        <label>Status</label>
        <label class="toggle-row">
          <input type="checkbox" bind:checked={draft.is_closed} />
          <span>Closed</span>
        </label>
      </div>
    </div>

    <div class="field">
      <label>Project <span class="req">*</span></label>
      <select
        value={String(draft.project_id)}
        on:change={e => {
          draft = { ...draft, project_id: Number((e.target as HTMLSelectElement).value) };
        }}
      >
        {#each allProjects as p}
          <option value={String(p.id)}>{p.title}</option>
        {/each}
      </select>
    </div>

    <div class="section-title">Categories</div>

    <div class="cats-grid">
      {#each cats as cat}
        {@const sel = draft[`category${cat.slot}_ids`]}
        {@const color = CAT_COLORS[`category_${cat.slot}`]?.hex ?? '#888'}
        <div class="cat-field">
          <div class="cat-field-label" style="color:{color}">{cat.label}</div>
          <!-- svelte-ignore a11y-no-static-element-interactions -->
          <div class="cat-badges"
            on:mousedown={() => { isClicking[cat.slot] = true; }}
            on:mouseup={() => { isClicking[cat.slot] = false; }}
          >
            {#if showAddInput[cat.slot]}
              <input
                class="inline-add-input"
                style="border-color:{color}; outline-color:{color}"
                bind:value={addDraft[cat.slot]}
                placeholder="Search or add…"
                on:blur={() => { if (!isClicking[cat.slot]) { addDraft[cat.slot] = ''; showAddInput[cat.slot] = false; } }}
                on:keydown={e => { if (e.key === 'Enter') { e.preventDefault(); addAndSelect(cat.slot, cat.vals); } if (e.key === 'Escape' || e.key === 'Tab') { isClicking[cat.slot] = false; addDraft[cat.slot] = ''; showAddInput[cat.slot] = false; } }}
                autofocus
              />
            {:else}
              <button class="inline-add-btn" style="--cat-color:{color}" on:click={() => showAddInput[cat.slot] = true}>Add or search</button>
            {/if}
            {#each filteredVals(cat.vals, addDraft[cat.slot]) as v (v.id)}
              <Badge
                label={v.label}
                catType="category_{cat.slot}"
                selected={sel.includes(v.id)}
                on:click={() => toggleCat(cat.slot, v.id)}
                size="sm"
              />
            {/each}
            {#if cat.vals.length === 0 && !showAddInput[cat.slot]}
              <span class="no-vals">No values</span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </div>

  <div class="panel-footer">
    {#if !isNew}
      <button class="btn-danger" on:click={remove}>
        {confirmDelete ? 'Confirm delete?' : 'Delete'}
      </button>
    {/if}
    <div class="spacer"></div>
    <button class="btn-secondary" on:click={close}>Cancel</button>
    <button class="btn-primary" on:click={save} disabled={!draft.title.trim() || !draft.project_id}>
      {isNew ? 'Create' : 'Save'}
    </button>
  </div>
</div>

<style>
  .backdrop {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.4);
    backdrop-filter: blur(2px);
    z-index: 100;
  }

  .panel {
    position: fixed;
    top: 0; right: 0; bottom: 0;
    width: 780px;
    max-width: 100vw;
    background: var(--surface);
    border-left: 1px solid var(--border);
    z-index: 101;
    display: flex;
    flex-direction: column;
    box-shadow: -8px 0 40px rgba(0,0,0,0.15);
    animation: slideIn 0.2s ease;
  }

  @keyframes slideIn {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }

  .panel-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: var(--sp-panel-gap, 20px) var(--sp-panel-pad, 24px);
    border-bottom: 1px solid var(--border);
  }
  h2 { margin: 0; font-size: 18px; font-weight: 700; color: var(--text); }

  .icon-btn {
    width: 30px; height: 30px;
    border: none; background: none;
    color: var(--text-muted); cursor: pointer;
    border-radius: 6px; font-size: 16px;
    display: flex; align-items: center; justify-content: center;
    transition: background 0.15s;
  }
  .icon-btn:hover { background: var(--surface-2); color: var(--text); }

  .panel-body {
    flex: 1 1 auto; overflow-y: auto;
    padding: calc(var(--sp-panel-pad, 24px) + 4px) calc(var(--sp-panel-pad, 24px) + 8px);
    display: flex; flex-direction: column; gap: var(--sp-panel-gap, 16px);
  }

  .field { display: flex; flex-direction: column; gap: var(--sp-field-gap, 5px); flex: 0 0 auto; }
  .field-row { display: flex; gap: 12px; }
  .field-row .field { flex: 1; }

  label {
    font-size: 12px; font-weight: 600;
    color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.06em;
  }
  .req { color: var(--accent); }

  input, textarea, select {
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 9px 12px;
    color: var(--text);
    font-size: 14px;
    font-family: inherit;
    outline: none;
    transition: border-color 0.15s;
    width: 100%;
    box-sizing: border-box;
  }
  input:focus, textarea:focus, select:focus { border-color: var(--accent); }
  textarea { resize: vertical; min-height: 90px; }

  .toggle-row {
    display: flex; align-items: center; gap: 8px;
    font-size: 14px; font-weight: 400; text-transform: none; letter-spacing: 0;
    color: var(--text); cursor: pointer; padding-top: 8px;
  }
  .toggle-row input { width: auto; }

  .section-title {
    font-size: 12px; font-weight: 700;
    text-transform: uppercase; letter-spacing: 0.08em;
    color: var(--text-muted);
    border-top: 1px solid var(--border);
    padding-top: 12px;
    margin-bottom: -4px;
  }

  .cats-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }

  .cat-field { display: flex; flex-direction: column; gap: 4px; }

  .cat-field-label {
    font-size: 10px; font-weight: 700;
    text-transform: uppercase; letter-spacing: 0.08em;
    opacity: 0.85;
  }

  .cat-badges {
    display: flex; flex-wrap: wrap; gap: 4px;
    padding: 6px 8px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 8px;
    min-height: 32px;
  }

  .no-vals {
    font-size: 11px; color: var(--text-muted);
    font-style: italic; align-self: center;
  }

  .inline-add-btn {
    --cat-color: var(--text-muted);
    font-size: 11px; color: var(--cat-color);
    background: none; border: 1.5px dashed var(--cat-color);
    border-radius: 999px; padding: 2px 8px;
    cursor: pointer; font-family: inherit;
    transition: opacity 0.15s;
    align-self: center;
  }
  .inline-add-btn:hover { opacity: 0.7; }

  .inline-add-input {
    font-size: 11px; border: 1.5px solid var(--accent);
    border-radius: 999px; padding: 2px 8px;
    background: var(--surface); color: var(--text);
    outline: none; width: 90px; font-family: inherit;
  }

  .panel-footer {
    display: flex; align-items: center; gap: 10px;
    padding: var(--sp-field-gap, 16px) var(--sp-panel-pad, 24px);
    border-top: 1px solid var(--border);
  }
  .spacer { flex: 1; }

  .btn-primary, .btn-secondary, .btn-danger {
    padding: 8px 18px; border-radius: 8px;
    font-size: 14px; font-weight: 600;
    cursor: pointer; border: none;
    font-family: inherit; transition: opacity 0.15s, transform 0.1s;
  }
  .btn-primary:active, .btn-secondary:active, .btn-danger:active { transform: scale(0.97); }
  .btn-primary { background: var(--accent); color: #fff; }
  .btn-primary:hover { opacity: 0.9; }
  .btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-secondary { background: var(--surface-2); color: var(--text); border: 1px solid var(--border); }
  .btn-secondary:hover { background: var(--surface-3); }
  .btn-danger { background: transparent; color: #ef4444; border: 1px solid #ef4444; }
  .btn-danger:hover { background: rgba(239,68,68,0.1); }
</style>
