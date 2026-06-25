<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { ProjectLink } from '../types';
  import { createProjectLink, updateProjectLink, deleteProjectLink } from '../store';

  export let link: ProjectLink | null = null;
  export let projectId: number;
  export let projectTitle: string = '';

  const dispatch = createEventDispatcher();
  const isNew = !link || link.id === 0;

  let draft = {
    label: link?.label ?? '',
    url: link?.url ?? '',
  };

  let confirmDelete = false;
  let error = '';

  async function save() {
    const label = draft.label.trim();
    const url = draft.url.trim();
    if (!label || !url) return;
    error = '';
    try {
      if (isNew) {
        await createProjectLink({ project_id: projectId, label, url });
      } else {
        await updateProjectLink({ id: link!.id, project_id: projectId, label, url });
      }
      dispatch('close');
    } catch (e: any) { error = String(e); }
  }

  async function remove() {
    if (!confirmDelete) { confirmDelete = true; return; }
    error = '';
    try {
      await deleteProjectLink(link!.id);
      dispatch('close');
    } catch (e: any) { error = String(e); confirmDelete = false; }
  }

  function close() { dispatch('close'); }
</script>

<div class="backdrop" on:click={close} on:keydown={e => e.key === 'Escape' && close()} role="presentation"></div>

<div class="panel" role="dialog" aria-modal="true">
  <div class="panel-header">
    <div>
      <h2>{isNew ? 'New Link' : 'Edit Link'}</h2>
      {#if projectTitle}
        <p class="subtitle">{projectTitle}</p>
      {/if}
    </div>
    <button class="icon-btn" on:click={close} aria-label="Close">✕</button>
  </div>

  <div class="panel-body">
    {#if error}
      <p class="error">{error}</p>
    {/if}

    <div class="field">
      <label>Label <span class="req">*</span></label>
      <input bind:value={draft.label} placeholder="e.g. Project board" autofocus />
    </div>

    <div class="field">
      <label>URL <span class="req">*</span></label>
      <input bind:value={draft.url} placeholder="https://…"
        on:keydown={e => e.key === 'Enter' && save()}
      />
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
    <button class="btn-primary" on:click={save} disabled={!draft.label.trim() || !draft.url.trim()}>
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
    width: 400px; max-width: 100vw;
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
    display: flex; align-items: flex-start; justify-content: space-between;
    padding: 20px 24px; border-bottom: 1px solid var(--border);
  }
  h2 { margin: 0; font-size: 18px; font-weight: 700; color: var(--text); }
  .subtitle { margin: 3px 0 0; font-size: 12px; color: var(--text-muted); }
  .icon-btn {
    width: 30px; height: 30px; border: none; background: none;
    color: var(--text-muted); cursor: pointer; border-radius: 6px; font-size: 16px;
    display: flex; align-items: center; justify-content: center; transition: background 0.15s;
    flex-shrink: 0;
  }
  .icon-btn:hover { background: var(--surface-2); color: var(--text); }
  .panel-body {
    flex: 1; overflow-y: auto; padding: 24px;
    display: flex; flex-direction: column; gap: 16px;
  }
  .field { display: flex; flex-direction: column; gap: 5px; }
  label { font-size: 12px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; letter-spacing: 0.06em; }
  .req { color: var(--accent); }
  input {
    background: var(--surface-2); border: 1px solid var(--border); border-radius: 8px;
    padding: 9px 12px; color: var(--text); font-size: 14px; font-family: inherit;
    outline: none; transition: border-color 0.15s; width: 100%; box-sizing: border-box;
  }
  input:focus { border-color: var(--accent); }
  .error { font-size: 13px; color: #ef4444; background: rgba(239,68,68,0.08); border-radius: 6px; padding: 8px 12px; margin: 0; }
  .panel-footer { display: flex; align-items: center; gap: 10px; padding: 16px 24px; border-top: 1px solid var(--border); }
  .spacer { flex: 1; }
  .btn-primary, .btn-secondary, .btn-danger {
    padding: 8px 18px; border-radius: 8px; font-size: 14px; font-weight: 600;
    cursor: pointer; border: none; font-family: inherit; transition: opacity 0.15s;
  }
  .btn-primary { background: var(--accent); color: #fff; }
  .btn-primary:hover { opacity: 0.9; }
  .btn-primary:disabled { opacity: 0.4; cursor: not-allowed; }
  .btn-secondary { background: var(--surface-2); color: var(--text); border: 1px solid var(--border); }
  .btn-secondary:hover { background: var(--surface-3); }
  .btn-danger { background: transparent; color: #ef4444; border: 1px solid #ef4444; }
  .btn-danger:hover { background: rgba(239,68,68,0.1); }
</style>
