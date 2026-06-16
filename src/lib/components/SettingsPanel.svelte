<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { get } from 'svelte/store';
  import { settings, picklists, projects, saveSettings, createPicklistValue, updatePicklistValue, deletePicklistValue, createProject, updateProject, deleteProject } from '../store';
  import { CAT_COLORS } from '../types';
  import type { PicklistValue, Project, Density } from '../types';

  const dispatch = createEventDispatcher();

  // Local copies for editing
  let cat1Label = $settings.category1_label;
  let cat2Label = $settings.category2_label;
  let cat3Label = $settings.category3_label;
  let cat4Label = $settings.category4_label;
  let darkMode  = $settings.dark_mode;
  let density: Density = $settings.density;

  const DENSITY_OPTIONS: { value: Density; label: string }[] = [
    { value: 'compact', label: 'Compact' },
    { value: 'normal', label: 'Normal' },
    { value: 'comfortable', label: 'Comfortable' },
  ];

  const SECTIONS = [
    { id: 'appearance', label: 'Appearance' },
    { id: 'categories', label: 'Category Labels' },
    { id: 'logtypes', label: 'Log Types' },
    { id: 'projects', label: 'Projects' },
  ] as const;
  let activeSection: typeof SECTIONS[number]['id'] = 'appearance';

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
      density,
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

<div class="modal" role="dialog" aria-modal="true">
  <div class="modal-header">
    <h2>Settings</h2>
    <button class="icon-btn" on:click={close} aria-label="Close">✕</button>
  </div>

  <div class="modal-body">
    <nav class="settings-nav">
      {#each SECTIONS as s}
        <button
          class="nav-item"
          class:active={activeSection === s.id}
          on:click={() => activeSection = s.id}
        >{s.label}</button>
      {/each}
    </nav>

    <div class="settings-content">

      {#if activeSection === 'appearance'}
        <div class="content-section">
          <h3 class="content-title">Appearance</h3>
          <p class="content-hint">Visual preferences for the whole application.</p>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Dark mode</span>
              <span class="setting-desc">Switch between light and dark color schemes.</span>
            </div>
            <div class="toggle-track" class:on={darkMode} on:click={() => { darkMode = !darkMode; persistSettings(); }} on:keydown role="switch" aria-checked={darkMode} tabindex="0">
              <div class="toggle-thumb"></div>
            </div>
          </div>

          <div class="setting-row">
            <div class="setting-info">
              <span class="setting-label">Layout density</span>
              <span class="setting-desc">Controls padding and spacing across the app, settings, and log/project editors.</span>
            </div>
            <select class="density-select" bind:value={density} on:change={persistSettings}>
              {#each DENSITY_OPTIONS as opt}
                <option value={opt.value}>{opt.label}</option>
              {/each}
            </select>
          </div>
        </div>
      {/if}

      {#if activeSection === 'categories'}
        <div class="content-section">
          <h3 class="content-title">Category Labels</h3>
          <p class="content-hint">Rename the 4 customizable category slots used on logs and projects.</p>

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
        </div>
      {/if}

      {#if activeSection === 'logtypes'}
        <div class="content-section">
          <h3 class="content-title">Log Types</h3>
          <p class="content-hint">Define the types of logs you can create (e.g. Task, Decision, Event). Deleting a type is blocked while logs use it.</p>

          {#if deleteError}
            <p class="delete-error">{deleteError}</p>
          {/if}
          <div class="item-list">
            {#each logTypes as lt (lt.id)}
              <div class="item-row">
                {#if editingId === lt.id}
                  <input
                    class="inline-input"
                    bind:value={editingLabel}
                    on:blur={commitEdit}
                    on:keydown={e => { if (e.key === 'Enter') commitEdit(); if (e.key === 'Escape') editingId = null; }}
                    autofocus
                  />
                {:else}
                  <span class="item-label" on:click={() => startEdit(lt)} on:keydown role="button" tabindex="0">{lt.label}</span>
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
        </div>
      {/if}

      {#if activeSection === 'projects'}
        <div class="content-section">
          <h3 class="content-title">Projects</h3>
          <p class="content-hint">Manage projects and their hierarchy. Deleting a project is blocked while it has sub-projects.</p>

          {#if projectError}
            <p class="delete-error">{projectError}</p>
          {/if}
          <div class="item-list">
            {#each $projects as p (p.id)}
              <div class="item-row">
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
                  <span class="item-label" on:click={() => startEditProject(p)} on:keydown role="button" tabindex="0">
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
        </div>
      {/if}

    </div>
  </div>
</div>

<style>
  .backdrop {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.5);
    backdrop-filter: blur(2px);
    z-index: 100;
  }

  .modal {
    position: fixed;
    top: 10vh; left: 10vw;
    width: 80vw; height: 80vh;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 16px;
    z-index: 101;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 20px 60px rgba(0,0,0,0.25);
    animation: popIn 0.18s ease;
  }

  @keyframes popIn {
    from { opacity: 0; transform: scale(0.97) translateY(8px); }
    to   { opacity: 1; transform: scale(1) translateY(0); }
  }

  .modal-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: var(--sp-panel-gap, 20px) var(--sp-panel-pad, 24px);
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

  .modal-body {
    flex: 1;
    display: flex;
    overflow: hidden;
    min-height: 0;
  }

  .settings-nav {
    width: 200px; flex-shrink: 0;
    background: var(--surface-2);
    border-right: 1px solid var(--border);
    display: flex; flex-direction: column;
    padding: var(--sp-panel-gap, 12px);
    gap: 2px;
    overflow-y: auto;
  }

  .nav-item {
    text-align: left;
    background: none; border: none;
    padding: 10px 14px;
    border-radius: 8px;
    font-size: 14px; font-weight: 500;
    color: var(--text-muted);
    cursor: pointer;
    font-family: inherit;
    transition: background 0.15s, color 0.15s;
  }
  .nav-item:hover { background: var(--surface-3); color: var(--text); }
  .nav-item.active { background: var(--accent); color: #fff; font-weight: 600; }

  .settings-content {
    flex: 1;
    overflow-y: auto;
    padding: var(--sp-panel-pad, 32px);
  }

  .content-section {
    display: flex; flex-direction: column; gap: var(--sp-panel-gap, 16px);
    max-width: 640px;
  }

  .content-title {
    margin: 0;
    font-size: 20px; font-weight: 700;
    color: var(--text);
  }
  .content-hint {
    margin: -8px 0 4px;
    font-size: 13px; color: var(--text-muted); line-height: 1.5;
  }

  .setting-row {
    display: flex; align-items: center; justify-content: space-between; gap: 16px;
    padding: var(--sp-panel-gap, 14px) 0;
    border-bottom: 1px solid var(--border);
  }
  .setting-row:last-child { border-bottom: none; }
  .setting-info { display: flex; flex-direction: column; gap: 3px; }
  .setting-label { font-size: 14px; font-weight: 600; color: var(--text); }
  .setting-desc { font-size: 12px; color: var(--text-muted); max-width: 380px; }

  /* Toggle */
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

  .density-select {
    background: var(--surface-2); border: 1px solid var(--border);
    border-radius: 8px; padding: 7px 12px;
    color: var(--text); font-size: 13px; font-family: inherit;
    outline: none; cursor: pointer;
  }
  .density-select:focus { border-color: var(--accent); }

  /* Category labels */
  .label-grid { display: flex; flex-direction: column; gap: 10px; }
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
    padding: 8px 12px;
    color: var(--text);
    font-size: 14px;
    font-family: inherit;
    outline: none;
    transition: border-color 0.15s;
  }
  .label-row input:focus { border-color: var(--accent); }

  /* Log types / Projects shared list styles */
  .item-list { display: flex; flex-direction: column; gap: 6px; }

  .item-row {
    display: flex; align-items: center; gap: 8px;
    background: var(--surface-2);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 9px 14px;
  }

  .item-label {
    flex: 1; font-size: 14px; color: var(--text);
    cursor: pointer;
  }
  .item-label:hover { color: var(--accent); }

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
    padding: 9px 14px;
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
    flex-shrink: 0; width: 130px;
    background: var(--surface); border: 1px solid var(--border);
    border-radius: 6px; padding: 4px 6px;
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
    padding: 9px 14px;
  }

  .delete-error {
    margin: 0;
    font-size: 12px;
    color: #ef4444;
    background: rgba(239,68,68,0.08);
    border-radius: 6px;
    padding: 6px 10px;
  }
</style>
