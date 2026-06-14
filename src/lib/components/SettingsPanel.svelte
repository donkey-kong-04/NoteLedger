<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { get } from 'svelte/store';
  import { settings, picklists, projects, saveSettings, createPicklistValue, updatePicklistValue, deletePicklistValue, createProject, updateProject, deleteProject } from '../store';
  import { CAT_COLORS } from '../types';
  import type { PicklistValue, Project } from '../types';

  const dispatch = createEventDispatcher();

  // Local copies for editing
  let cat1Label = $settings.category1_label;
  let cat2Label = $settings.category2_label;
  let cat3Label = $settings.category3_label;
  let cat4Label = $settings.category4_label;
  let darkMode  = $settings.dark_mode;

  $: logTypes = $picklists.filter(v => v.picklist_type === 'log_type');

  // Log type editing state
  let newLogTypeInput = '';
  let showNewLogType = false;
  let editingId: number | null = null;
  let editingLabel = '';
  let deleteError = '';

  async function persistSettings() {
    await saveSettings({
      category1_label: cat1Label,
      category2_label: cat2Label,
      category3_label: cat3Label,
      category4_label: cat4Label,
      dark_mode: darkMode,
    });
  }

  async function addLogType() {
    const trimmed = newLogTypeInput.trim();
    newLogTypeInput = '';
    showNewLogType = false;
    if (!trimmed) return;
    await createPicklistValue('log_type', trimmed);
  }

  function startEdit(val: PicklistValue) {
    editingId = val.id;
    editingLabel = val.label;
  }

  async function commitEdit() {
    if (editingId === null) return;
    const trimmed = editingLabel.trim();
    const id = editingId;
    editingId = null;
    if (!trimmed) return;
    await updatePicklistValue(id, trimmed);
  }

  async function removeLogType(id: number) {
    try {
      await deletePicklistValue(id);
      deleteError = '';
    } catch (e: any) {
      deleteError = String(e);
    }
  }

  // ── Projects ──
  let newProjectInput = '';
  let showNewProject = false;
  let editingProjectId: number | null = null;
  let editingProjectTitle = '';
  let editingProjectParent: number | null = null;
  let projectError = '';

  async function addProject() {
    const trimmed = newProjectInput.trim();
    newProjectInput = '';
    showNewProject = false;
    if (!trimmed) return;
    try {
      await createProject({ title: trimmed, description: '', parent_id: null });
      projectError = '';
    } catch (e: any) { projectError = String(e); }
  }

  function startEditProject(p: Project) {
    editingProjectId = p.id;
    editingProjectTitle = p.title;
    editingProjectParent = p.parent_id;
  }

  async function commitEditProject() {
    if (editingProjectId === null) return;
    const id = editingProjectId;
    const title = editingProjectTitle.trim();
    const parent_id = editingProjectParent;
    editingProjectId = null;
    if (!title) return;
    try {
      const existing = $projects.find(p => p.id === id)!;
      await updateProject({ ...existing, title, parent_id });
      projectError = '';
    } catch (e: any) { projectError = String(e); }
  }

  async function removeProject(id: number) {
    try {
      await deleteProject(id);
      projectError = '';
    } catch (e: any) { projectError = String(e); }
  }

  function projectLabel(p: Project): string {
    if (p.parent_id === null) return p.title;
    const parent = $projects.find(x => x.id === p.parent_id);
    return parent ? `${parent.title} › ${p.title}` : p.title;
  }

  function close() { dispatch('close'); }
</script>

<div class="backdrop" on:click={close} on:keydown={e => e.key === 'Escape' && close()} role="presentation"></div>

<div class="panel" role="dialog" aria-modal="true">
  <div class="panel-header">
    <h2>Settings</h2>
    <button class="icon-btn" on:click={close} aria-label="Close">✕</button>
  </div>

  <div class="panel-body">

    <!-- Appearance -->
    <section>
      <div class="section-title">Appearance</div>
      <label class="toggle-row">
        <div class="toggle-track" class:on={darkMode} on:click={() => { darkMode = !darkMode; persistSettings(); }} on:keydown role="switch" aria-checked={darkMode} tabindex="0">
          <div class="toggle-thumb"></div>
        </div>
        <span>Dark mode</span>
      </label>
    </section>

    <!-- Category Labels -->
    <section>
      <div class="section-title">Category Labels</div>
      <div class="label-grid">
        {#each [
          { key: 'category_1', bind: () => cat1Label, set: (v) => { cat1Label = v; } },
          { key: 'category_2', bind: () => cat2Label, set: (v) => { cat2Label = v; } },
          { key: 'category_3', bind: () => cat3Label, set: (v) => { cat3Label = v; } },
          { key: 'category_4', bind: () => cat4Label, set: (v) => { cat4Label = v; } },
        ] as cat, i}
          <div class="label-row">
            <span class="cat-dot" style="background: {CAT_COLORS[cat.key]?.hex}"></span>
            <input
              value={[cat1Label, cat2Label, cat3Label, cat4Label][i]}
              on:input={e => { cat.set((e.target as HTMLInputElement).value); }}
              on:blur={persistSettings}
              on:keydown={e => e.key === 'Enter' && persistSettings()}
            />
          </div>
        {/each}
      </div>
    </section>

    <!-- Log Types -->
    <section>
      <div class="section-title">Log Types</div>
      {#if deleteError}
        <p class="delete-error">{deleteError}</p>
      {/if}
      <div class="log-types-list">
        {#each logTypes as lt (lt.id)}
          <div class="log-type-row">
            {#if editingId === lt.id}
              <input
                class="inline-input"
                bind:value={editingLabel}
                on:blur={commitEdit}
                on:keydown={e => { if (e.key === 'Enter') commitEdit(); if (e.key === 'Escape') editingId = null; }}
                autofocus
              />
            {:else}
              <span class="log-type-label" on:click={() => startEdit(lt)} on:keydown role="button" tabindex="0">{lt.label}</span>
              <button class="del-btn" on:click={() => removeLogType(lt.id)} title="Delete">✕</button>
            {/if}
          </div>
        {/each}

        {#if showNewLogType}
          <input
            class="inline-input"
            bind:value={newLogTypeInput}
            placeholder="New log type…"
            on:blur={() => { showNewLogType = false; newLogTypeInput = ''; }}
            on:keydown={e => { if (e.key === 'Enter') { e.preventDefault(); addLogType(); } if (e.key === 'Escape') { showNewLogType = false; newLogTypeInput = ''; } }}
            autofocus
          />
        {:else}
          <button class="add-btn" on:click={() => showNewLogType = true}>+ Add log type</button>
        {/if}
      </div>
    </section>

    <!-- Projects -->
    <section>
      <div class="section-title">Projects</div>
      {#if projectError}
        <p class="delete-error">{projectError}</p>
      {/if}
      <div class="log-types-list">
        {#each $projects as p (p.id)}
          <div class="log-type-row">
            {#if editingProjectId === p.id}
              <div class="project-edit">
                <input
                  class="inline-input"
                  bind:value={editingProjectTitle}
                  placeholder="Project name"
                  on:keydown={e => { if (e.key === 'Enter') commitEditProject(); if (e.key === 'Escape') editingProjectId = null; }}
                />
                <select class="parent-select" bind:value={editingProjectParent}
                  on:blur={commitEditProject}>
                  <option value={null}>No parent</option>
                  {#each $projects.filter(x => x.id !== p.id) as opt}
                    <option value={opt.id}>{opt.title}</option>
                  {/each}
                </select>
                <button class="save-btn" on:click={commitEditProject}>✓</button>
              </div>
            {:else}
              <span class="log-type-label" on:click={() => startEditProject(p)} on:keydown role="button" tabindex="0">
                {projectLabel(p)}
              </span>
              <button class="del-btn" on:click={() => removeProject(p.id)} title="Delete">✕</button>
            {/if}
          </div>
        {/each}

        {#if showNewProject}
          <input
            class="inline-input standalone"
            bind:value={newProjectInput}
            placeholder="New project name…"
            on:blur={() => { showNewProject = false; newProjectInput = ''; }}
            on:keydown={e => { if (e.key === 'Enter') { e.preventDefault(); addProject(); } if (e.key === 'Escape') { showNewProject = false; newProjectInput = ''; } }}
            autofocus
          />
        {:else}
          <button class="add-btn" on:click={() => showNewProject = true}>+ Add project</button>
        {/if}
      </div>
    </section>

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
    width: 380px;
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
    to   { transform: translateX(0);    opacity: 1; }
  }

  .panel-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 20px 24px;
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
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
    flex: 1; overflow-y: auto;
    padding: 24px;
    display: flex; flex-direction: column; gap: 28px;
  }

  section { display: flex; flex-direction: column; gap: 12px; }

  .section-title {
    font-size: 11px; font-weight: 700;
    text-transform: uppercase; letter-spacing: 0.08em;
    color: var(--text-muted);
    padding-bottom: 4px;
    border-bottom: 1px solid var(--border);
  }

  /* Toggle */
  .toggle-row {
    display: flex; align-items: center; gap: 12px;
    cursor: pointer; font-size: 14px; color: var(--text);
  }
  .toggle-track {
    width: 40px; height: 22px;
    background: var(--border);
    border-radius: 999px;
    position: relative;
    cursor: pointer;
    transition: background 0.2s;
    flex-shrink: 0;
  }
  .toggle-track.on { background: var(--accent); }
  .toggle-thumb {
    position: absolute;
    top: 3px; left: 3px;
    width: 16px; height: 16px;
    background: #fff;
    border-radius: 50%;
    transition: transform 0.2s;
    box-shadow: 0 1px 3px rgba(0,0,0,0.2);
  }
  .toggle-track.on .toggle-thumb { transform: translateX(18px); }

  /* Category labels */
  .label-grid { display: flex; flex-direction: column; gap: 8px; }
  .label-row { display: flex; align-items: center; gap: 10px; }
  .cat-dot {
    width: 10px; height: 10px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .label-row input {
    flex: 1;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 7px 10px;
    color: var(--text);
    font-size: 14px;
    font-family: inherit;
    outline: none;
    transition: border-color 0.15s;
  }
  .label-row input:focus { border-color: var(--accent); }

  /* Log types */
  .log-types-list { display: flex; flex-direction: column; gap: 6px; }

  .log-type-row {
    display: flex; align-items: center; gap: 8px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 8px 12px;
  }

  .log-type-label {
    flex: 1; font-size: 14px; color: var(--text);
    cursor: pointer;
  }
  .log-type-label:hover { color: var(--accent); }

  .del-btn {
    background: none; border: none;
    color: var(--text-muted); cursor: pointer;
    font-size: 12px; padding: 2px 4px;
    border-radius: 4px;
    transition: color 0.15s, background 0.15s;
  }
  .del-btn:hover { color: #ef4444; background: rgba(239,68,68,0.1); }

  .inline-input {
    flex: 1;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text);
    font-size: 14px;
    font-family: inherit;
  }

  .add-btn {
    font-size: 13px; color: var(--text-muted);
    background: none;
    border: 1.5px dashed var(--border);
    border-radius: 8px;
    padding: 8px 12px;
    cursor: pointer;
    text-align: left;
    font-family: inherit;
    transition: color 0.15s, border-color 0.15s;
  }
  .add-btn:hover { color: var(--text); border-color: var(--text-muted); }

  .project-edit {
    flex: 1; display: flex; align-items: center; gap: 6px; min-width: 0;
  }
  .parent-select {
    flex-shrink: 0; width: 110px;
    background: var(--surface); border: 1px solid var(--border);
    border-radius: 6px; padding: 3px 6px;
    color: var(--text); font-size: 12px; font-family: inherit; outline: none;
  }
  .save-btn {
    background: none; border: none; color: var(--accent);
    cursor: pointer; font-size: 14px; padding: 2px 4px;
    border-radius: 4px; flex-shrink: 0;
  }
  .save-btn:hover { background: var(--surface-2); }

  .inline-input.standalone {
    width: 100%;
    background: var(--surface-2);
    border: 1px solid var(--accent);
    border-radius: 8px;
    padding: 8px 12px;
  }

  .delete-error {
    margin: 0 0 4px;
    font-size: 12px;
    color: #ef4444;
    background: rgba(239,68,68,0.08);
    border-radius: 6px;
    padding: 6px 10px;
  }
</style>
