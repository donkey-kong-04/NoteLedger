<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import ProjectCard from '$lib/components/ProjectCard.svelte';
  import LogEditor from '$lib/components/LogEditor.svelte';
  import ProjectEditor from '$lib/components/ProjectEditor.svelte';
  import LinkEditor from '$lib/components/LinkEditor.svelte';
  import { settings, picklists, projects, logs, projectLinks, loadAll, cloneProject, pendingProjectFocus } from '$lib/store';
  import type { Log, Project, ProjectLink } from '$lib/types';
  import { densityStyle } from '$lib/density';

  onMount(() => loadAll());

  let editorLog: Log | null = null;
  let showEditor = false;
  let editorProject: Project | null = null;
  let showProjectEditor = false;
  let editorLink: ProjectLink | null = null;
  let editorLinkProjectId: number = 0;
  let editorLinkProjectTitle: string = '';
  let showLinkEditor = false;

  // Fold/unfold all — same mechanism as the homepage.
  let allCollapsed = false;
  let collapseSignal = 0;
  function toggleFoldAll() {
    allCollapsed = !allCollapsed;
    collapseSignal++;
  }

  $: isDark = $settings.dark_mode;
  $: pageDensityStyle = densityStyle($settings.density);

  $: logTypes = $picklists.filter(v => v.picklist_type === 'log_type');
  const byLabel = (a: {label: string}, b: {label: string}) => a.label.localeCompare(b.label);
  $: cat1Vals = $picklists.filter(v => v.picklist_type === 'category_1').sort(byLabel);
  $: cat2Vals = $picklists.filter(v => v.picklist_type === 'category_2').sort(byLabel);
  $: cat3Vals = $picklists.filter(v => v.picklist_type === 'category_3').sort(byLabel);
  $: cat4Vals = $picklists.filter(v => v.picklist_type === 'category_4').sort(byLabel);

  // Only template projects live here — no filters on this page, everything visible.
  $: templateProjects = $projects.filter(p => p.is_template);
  $: topLevelTemplates = templateProjects.filter(p => p.parent_id == null);
  $: allTemplateIds = new Set(templateProjects.map(p => p.id));

  // ── Editors (same behavior as the homepage) ───────────────────────────────
  function openEdit(e: CustomEvent<Log>) { editorLog = e.detail; showEditor = true; }
  function closeEditor() { showEditor = false; editorLog = null; }

  function openNewProject() { editorProject = null; showProjectEditor = true; }
  function openEditProject(e: CustomEvent<Project>) { editorProject = e.detail; showProjectEditor = true; }
  function closeProjectEditor() { showProjectEditor = false; editorProject = null; }

  function openNewLink(e: CustomEvent<Project>) {
    editorLink = null;
    editorLinkProjectId = e.detail.id;
    editorLinkProjectTitle = e.detail.title;
    showLinkEditor = true;
  }
  function openEditLink(e: CustomEvent<ProjectLink>) {
    editorLink = e.detail;
    const proj = templateProjects.find(p => p.id === e.detail.project_id);
    editorLinkProjectId = e.detail.project_id;
    editorLinkProjectTitle = proj?.title ?? '';
    showLinkEditor = true;
  }
  function closeLinkEditor() { showLinkEditor = false; editorLink = null; }

  function openNewLogInProject(e: CustomEvent<{ project: Project; typeId: number }>) {
    editorLog = { type_id: e.detail.typeId, project_id: e.detail.project.id } as any;
    showEditor = true;
  }

  function openNewSubProject(e: CustomEvent<Project>) {
    const parent = e.detail;
    editorProject = { id: 0, title: '', description: '', parent_id: parent.id, is_closed: false, is_template: true, start_date: null, end_date: null, category1_ids: [], category2_ids: [], category3_ids: [], category4_ids: [] };
    showProjectEditor = true;
  }

  // ── Clone dialog ──────────────────────────────────────────────────────────
  let cloneSource: Project | null = null;
  let cloneTitle = '';
  let cloneError = '';
  let cloning = false;

  function openCloneDialog(e: CustomEvent<Project>) {
    cloneSource = e.detail;
    cloneTitle = e.detail.title;
    cloneError = '';
  }

  function closeCloneDialog() { cloneSource = null; cloneTitle = ''; cloneError = ''; }

  async function confirmClone() {
    if (!cloneSource || !cloneTitle.trim() || cloning) return;
    cloning = true;
    try {
      const newId = await cloneProject(cloneSource.id, cloneTitle.trim());
      closeCloneDialog();
      // Land on the homepage focused on the freshly created project.
      pendingProjectFocus.set(newId);
      await goto('/');
    } catch (e: any) {
      cloneError = String(e);
    } finally {
      cloning = false;
    }
  }
</script>

<div class="page" class:dark={isDark} style={pageDensityStyle}>
  <header class="page-header">
    <h1>Templates</h1>
    <span class="count">{topLevelTemplates.length} template{topLevelTemplates.length !== 1 ? 's' : ''}</span>
    <nav class="header-nav">
      <button class="btn-secondary-sm" on:click={toggleFoldAll}>{allCollapsed ? '▸ Unfold all' : '▾ Fold all'}</button>
      <button class="btn-secondary-sm" on:click={openNewProject}>+ New Template</button>
    </nav>
  </header>

  <main class="main">
    <div class="cards">
      {#each topLevelTemplates as project (project.id)}
        <ProjectCard
          {project}
          subProjects={templateProjects.filter(p => p.parent_id != null && Number(p.parent_id) === Number(project.id))}
          allLogs={$logs}
          allLogsTotal={$logs}
          allProjects={templateProjects}
          allLinks={$projectLinks}
          visibleProjectIds={allTemplateIds}
          ancestorOnlyProjectIds={new Set()}
          showClosed={true}
          {collapseSignal}
          collapseAll={allCollapsed}
          depth={0}
          showCloneButton={true}
          {logTypes} {cat1Vals} {cat2Vals} {cat3Vals} {cat4Vals}
          on:edit={openEdit}
          on:editProject={openEditProject}
          on:newSubProject={openNewSubProject}
          on:newLogInProject={openNewLogInProject}
          on:newLink={openNewLink}
          on:editLink={openEditLink}
          on:cloneProject={openCloneDialog}
        />
      {/each}

      {#if topLevelTemplates.length === 0}
        <div class="empty">
          <span class="empty-icon">📐</span>
          <p>No templates yet. Click <strong>+ New Template</strong> to create one, then clone it into real projects from here.</p>
        </div>
      {/if}
    </div>
  </main>

  {#if showEditor}
    <LogEditor
      log={editorLog}
      {logTypes} {cat1Vals} {cat2Vals} {cat3Vals} {cat4Vals}
      cat1Label={$settings.category1_label} cat2Label={$settings.category2_label}
      cat3Label={$settings.category3_label} cat4Label={$settings.category4_label}
      allProjects={templateProjects}
      on:close={closeEditor}
    />
  {/if}

  {#if showProjectEditor}
    <ProjectEditor
      project={editorProject}
      allProjects={templateProjects}
      isTemplate={true}
      {cat1Vals} {cat2Vals} {cat3Vals} {cat4Vals}
      cat1Label={$settings.category1_label} cat2Label={$settings.category2_label}
      cat3Label={$settings.category3_label} cat4Label={$settings.category4_label}
      on:close={closeProjectEditor}
    />
  {/if}

  {#if showLinkEditor}
    <LinkEditor
      link={editorLink}
      projectId={editorLinkProjectId}
      projectTitle={editorLinkProjectTitle}
      on:close={closeLinkEditor}
    />
  {/if}

  {#if cloneSource}
    <div class="backdrop" on:click={closeCloneDialog} on:keydown={e => e.key === 'Escape' && closeCloneDialog()} role="presentation"></div>
    <div class="clone-dialog" role="dialog" aria-modal="true">
      <h2>Clone template</h2>
      <p class="clone-hint">Creates an independent copy of <strong>{cloneSource.title}</strong> — including its logs, links, and sub-projects — as a regular project on the homepage. Later changes to the template won't affect it.</p>
      <label class="clone-label" for="clone-title">New project title</label>
      <input
        id="clone-title"
        class="clone-input"
        bind:value={cloneTitle}
        on:keydown={e => { if (e.key === 'Enter') confirmClone(); if (e.key === 'Escape') closeCloneDialog(); }}
        autofocus
      />
      {#if cloneError}
        <p class="clone-error">{cloneError}</p>
      {/if}
      <div class="clone-actions">
        <button class="btn-secondary-sm" on:click={closeCloneDialog}>Cancel</button>
        <button class="btn-primary" disabled={!cloneTitle.trim() || cloning} on:click={confirmClone}>
          {cloning ? 'Cloning…' : '⧉ Clone'}
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .page {
    display: flex; flex-direction: column;
    height: 100%; overflow: hidden;
    background: var(--surface-2); color: var(--text);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  }

  .page-header {
    display: flex; align-items: center; gap: 16px;
    padding: 12px 20px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  h1 { margin: 0; font-size: 18px; font-weight: 700; }
  .count { font-size: 12px; color: var(--text-muted); }
  .header-nav { margin-left: auto; display: flex; align-items: center; gap: 10px; }

  .btn-secondary-sm {
    background: none; color: var(--text-muted);
    border: 1px solid var(--border); border-radius: 10px;
    padding: 8px 14px; font-size: 14px; font-weight: 600; cursor: pointer;
    font-family: inherit; transition: background 0.15s, color 0.15s;
  }
  .btn-secondary-sm:hover { background: var(--surface-2); color: var(--text); }

  .main {
    flex: 1; display: flex; flex-direction: column;
    overflow: hidden; padding: var(--sp-main-pad, 24px); min-height: 0;
  }
  .cards {
    flex: 1; overflow-y: auto;
    display: flex; flex-direction: column;
    gap: var(--sp-card-gap, 12px); padding-right: 4px;
  }

  .empty {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    gap: 12px; padding: 60px 20px; color: var(--text-muted); text-align: center;
  }
  .empty-icon { font-size: 48px; }
  .empty p { font-size: 15px; margin: 0; max-width: 420px; }

  /* Clone dialog */
  .backdrop {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.5);
    backdrop-filter: blur(2px);
    z-index: 100;
  }
  .clone-dialog {
    position: fixed;
    top: 50%; left: 50%;
    transform: translate(-50%, -50%);
    width: min(440px, 90vw);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 14px;
    padding: 22px 24px;
    z-index: 101;
    display: flex; flex-direction: column; gap: 10px;
    box-shadow: 0 20px 60px rgba(0,0,0,0.25);
  }
  .clone-dialog h2 { margin: 0; font-size: 16px; font-weight: 700; color: var(--text); }
  .clone-hint { margin: 0; font-size: 12px; color: var(--text-muted); line-height: 1.5; }
  .clone-label { font-size: 12px; font-weight: 600; color: var(--text-muted); }
  .clone-input {
    background: var(--surface-2); color: var(--text);
    border: 1px solid var(--border); border-radius: 8px;
    padding: 8px 12px; font-size: 14px; font-family: inherit;
    outline: none;
  }
  .clone-input:focus { border-color: var(--accent); }
  .clone-error {
    margin: 0; font-size: 12px; color: #ef4444;
    background: rgba(239,68,68,0.08); border-radius: 6px; padding: 6px 10px;
  }
  .clone-actions { display: flex; justify-content: flex-end; gap: 8px; margin-top: 4px; }
  .btn-primary {
    background: var(--accent); color: #fff; border: none; border-radius: 10px;
    padding: 8px 16px; font-size: 14px; font-weight: 600; cursor: pointer;
    font-family: inherit; transition: opacity 0.15s;
  }
  .btn-primary:hover { opacity: 0.9; }
  .btn-primary:disabled { opacity: 0.5; cursor: default; }
</style>
