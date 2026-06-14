<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { Project } from '../types';
  import { createProject, updateProject, deleteProject } from '../store';

  export let project: Project | null = null;
  export let allProjects: Project[];

  const dispatch = createEventDispatcher();
  const isNew = !project || project.id === 0;

  let draft: Project = project ? { ...project } : { id: 0, title: '', description: '', parent_id: null };
  let confirmDelete = false;
  let error = '';

  $: availableParents = allProjects.filter(p => p.id !== draft.id);

  async function save() {
    if (!draft.title.trim()) return;
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
    width: 420px; max-width: 100vw;
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
    padding: 20px 24px; border-bottom: 1px solid var(--border);
  }
  h2 { margin: 0; font-size: 18px; font-weight: 700; color: var(--text); }
  .icon-btn {
    width: 30px; height: 30px; border: none; background: none;
    color: var(--text-muted); cursor: pointer; border-radius: 6px; font-size: 16px;
    display: flex; align-items: center; justify-content: center; transition: background 0.15s;
  }
  .icon-btn:hover { background: var(--surface-2); color: var(--text); }
  .panel-body { flex: 1; overflow-y: auto; padding: 24px; display: flex; flex-direction: column; gap: 16px; }
  .field { display: flex; flex-direction: column; gap: 5px; }
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
  .panel-footer { display: flex; align-items: center; gap: 10px; padding: 16px 24px; border-top: 1px solid var(--border); }
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
