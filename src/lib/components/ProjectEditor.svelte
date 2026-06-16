<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import Badge from './Badge.svelte';
  import type { Project, PicklistValue } from '../types';
  import { CAT_COLORS } from '../types';
  import { createProject, updateProject, deleteProject, createPicklistValue, picklists } from '../store';

  export let project: Project | null = null;
  export let allProjects: Project[];
  export let cat1Vals: PicklistValue[] = [];
  export let cat2Vals: PicklistValue[] = [];
  export let cat3Vals: PicklistValue[] = [];
  export let cat4Vals: PicklistValue[] = [];
  export let cat1Label: string = 'Category 1';
  export let cat2Label: string = 'Category 2';
  export let cat3Label: string = 'Category 3';
  export let cat4Label: string = 'Category 4';

  const dispatch = createEventDispatcher();
  const isNew = !project || project.id === 0;

  let draft: Project = project ? {
    ...project,
    category1_ids: [...(project.category1_ids ?? [])],
    category2_ids: [...(project.category2_ids ?? [])],
    category3_ids: [...(project.category3_ids ?? [])],
    category4_ids: [...(project.category4_ids ?? [])],
  } : { id: 0, title: '', description: '', parent_id: null, category1_ids: [], category2_ids: [], category3_ids: [], category4_ids: [] };

  let confirmDelete = false;
  let error = '';

  $: availableParents = allProjects.filter(p => p.id !== draft.id);

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

  async function addAndSelect(slot: 1|2|3|4) {
    const trimmed = addDraft[slot].trim();
    if (!trimmed) { showAddInput[slot] = false; return; }
    addDraft[slot] = '';
    showAddInput[slot] = false;
    const p = createPicklistValue(`category_${slot}`, trimmed).then(newVal => {
      toggleCat(slot, newVal.id);
    });
    pendingAdd = p;
    await p;
  }

  function toggleCat(slot: 1|2|3|4, id: number) {
    const key = `category${slot}_ids` as const;
    const cur = draft[key];
    draft = { ...draft, [key]: cur.includes(id) ? cur.filter(x => x !== id) : [...cur, id] };
  }

  async function save() {
    if (!draft.title.trim()) return;
    await pendingAdd;
    error = '';
    try {
      if (isNew) { await createProject(draft); }
      else { await updateProject(draft); }
      dispatch('close');
    } catch (e: any) { error = String(e); }
  }

  async function remove() {
    if (!confirmDelete) { confirmDelete = true; return; }
    error = '';
    try {
      await deleteProject(draft.id);
      dispatch('close');
    } catch (e: any) { error = String(e); confirmDelete = false; }
  }

  function close() { dispatch('close'); }
</script>

<div class="backdrop" on:click={close} on:keydown={e => e.key === 'Escape' && close()} role="presentation"></div>

<div class="panel" role="dialog" aria-modal="true">
  <div class="panel-header">
    <h2>{isNew ? 'New Project' : 'Edit Project'}</h2>
    <button class="icon-btn" on:click={close} aria-label="Close">✕</button>
  </div>

  <div class="panel-body">
    {#if error}
      <p class="error">{error}</p>
    {/if}

    <div class="field">
      <label>Title <span class="req">*</span></label>
      <input bind:value={draft.title} placeholder="Project name" />
    </div>

    <div class="field">
      <label>Description</label>
      <textarea bind:value={draft.description} rows="3" placeholder="What is this project about?"></textarea>
    </div>

    <div class="field">
      <label>Parent project</label>
      <select bind:value={draft.parent_id}>
        <option value={null}>None (top-level)</option>
        {#each availableParents as p}
          <option value={p.id}>{p.title}</option>
        {/each}
      </select>
    </div>

    <div class="section-title">Default Categories</div>
    <p class="section-hint">Logs in this project will match filters for these categories even if not set on the log itself.</p>

    <div class="cats-grid">
      {#each cats as cat}
        {@const sel = draft[`category${cat.slot}_ids`]}
        {@const color = CAT_COLORS[`category_${cat.slot}`]?.hex ?? '#888'}
        <div class="cat-field">
          <div class="cat-field-label" style="color:{color}">{cat.label}</div>
          <div class="cat-badges">
            {#each cat.vals as v (v.id)}
              <Badge
                label={v.label}
                catType="category_{cat.slot}"
                selected={sel.includes(v.id)}
                on:click={() => toggleCat(cat.slot, v.id)}
                size="sm"
              />
            {/each}
            {#if showAddInput[cat.slot]}
              <input
                class="inline-add-input"
                style="border-color:{color}; outline-color:{color}"
                bind:value={addDraft[cat.slot]}
                placeholder="New value…"
                on:keydown={e => { if (e.key === 'Enter') { e.preventDefault(); addAndSelect(cat.slot); } if (e.key === 'Escape' || e.key === 'Tab') { addDraft[cat.slot] = ''; showAddInput[cat.slot] = false; } }}
                on:blur={() => { addDraft[cat.slot] = ''; showAddInput[cat.slot] = false; }}
                autofocus
              />
            {:else}
              <button class="inline-add-btn" style="--cat-color:{color}" on:click={() => showAddInput[cat.slot] = true}>+ Add</button>
            {/if}
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
    <button class="btn-primary" on:click={save} disabled={!draft.title.trim()}>
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
    position: fixed; top: 0; right: 0; bottom: 0;
    width: 480px; max-width: 100vw;
    background: var(--surface);
    border-left: 1px solid var(--border);
    z-index: 101;
    display: flex; flex-direction: column;
    box-shadow: -8px 0 40px rgba(0,0,0,0.15);
    animation: slideIn 0.2s ease;
  }
  @keyframes slideIn {
    from { transform: translateX(100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }
  .panel-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: var(--sp-panel-gap, 20px) var(--sp-panel-pad, 24px); border-bottom: 1px solid var(--border);
  }
  h2 { margin: 0; font-size: 18px; font-weight: 700; color: var(--text); }
  .icon-btn {
    width: 30px; height: 30px; border: none; background: none;
    color: var(--text-muted); cursor: pointer; border-radius: 6px; font-size: 16px;
    display: flex; align-items: center; justify-content: center; transition: background 0.15s;
  }
  .icon-btn:hover { background: var(--surface-2); color: var(--text); }
  .panel-body { flex: 1; overflow-y: auto; padding: var(--sp-panel-pad, 24px); display: flex; flex-direction: column; gap: var(--sp-panel-gap, 16px); }
  .field { display: flex; flex-direction: column; gap: var(--sp-field-gap, 5px); }
  label { font-size: 12px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.06em; }
  .req { color: var(--accent); }
  input, textarea, select {
    background: var(--surface-2); border: 1px solid var(--border); border-radius: 8px;
    padding: 9px 12px; color: var(--text); font-size: 14px; font-family: inherit;
    outline: none; transition: border-color 0.15s; width: 100%; box-sizing: border-box;
  }
  input:focus, textarea:focus, select:focus { border-color: var(--accent); }
  textarea { resize: vertical; min-height: 80px; }
  .error { font-size: 13px; color: #ef4444; background: rgba(239,68,68,0.08); border-radius: 6px; padding: 8px 12px; margin: 0; }

  .section-title {
    font-size: 12px; font-weight: 700;
    text-transform: uppercase; letter-spacing: 0.08em;
    color: var(--text-muted);
    border-top: 1px solid var(--border);
    padding-top: 12px;
    margin-bottom: -8px;
  }
  .section-hint {
    font-size: 12px; color: var(--text-muted); margin: 0; line-height: 1.5;
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
  .no-vals { font-size: 11px; color: var(--text-muted); font-style: italic; align-self: center; }

  .inline-add-btn {
    --cat-color: var(--text-muted);
    font-size: 11px; color: var(--cat-color);
    background: none; border: 1.5px dashed var(--cat-color);
    border-radius: 999px; padding: 2px 8px;
    cursor: pointer; font-family: inherit;
    transition: opacity 0.15s; align-self: center;
  }
  .inline-add-btn:hover { opacity: 0.7; }

  .inline-add-input {
    font-size: 11px; border: 1.5px solid var(--accent);
    border-radius: 999px; padding: 2px 8px;
    background: var(--surface); color: var(--text);
    outline: none; width: 90px; font-family: inherit;
  }

  .panel-footer { display: flex; align-items: center; gap: 10px; padding: var(--sp-field-gap, 16px) var(--sp-panel-pad, 24px); border-top: 1px solid var(--border); }
  .spacer { flex: 1; }
  .btn-primary, .btn-secondary, .btn-danger {
    padding: 8px 18px; border-radius: 8px; font-size: 14px; font-weight: 600;
    cursor: pointer; border: none; font-family: inherit; transition: opacity 0.15s, transform 0.1s;
  }
  .btn-primary { background: var(--accent); color: #fff; }
  .btn-primary:hover { opacity: 0.9; }
  .btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-secondary { background: var(--surface-2); color: var(--text); border: 1px solid var(--border); }
  .btn-secondary:hover { background: var(--surface-3); }
  .btn-danger { background: transparent; color: #ef4444; border: 1px solid #ef4444; }
  .btn-danger:hover { background: rgba(239,68,68,0.1); }
</style>
