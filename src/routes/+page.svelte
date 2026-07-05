<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import FilterPanel from '$lib/components/FilterPanel.svelte';
  import ProjectCard from '$lib/components/ProjectCard.svelte';
  import LogEditor from '$lib/components/LogEditor.svelte';
  import ProjectEditor from '$lib/components/ProjectEditor.svelte';
  import LinkEditor from '$lib/components/LinkEditor.svelte';
  import { settings, picklists, projects, logs, projectLinks, loadAll, pendingProjectFocus, showClosed, selCat1, selCat2, selCat3, selCat4, selProject, selLogType } from '$lib/store';
  import type { Log, Project, ProjectLink } from '$lib/types';
  import { logMatchesSlot, computeVisibility } from '$lib/filters';
  import { densityStyle as densityStyleFor } from '$lib/density';


  let editorLog: Log | null = null;
  let showEditor = false;
  let editorProject: Project | null = null;
  let showProjectEditor = false;
  let editorLink: ProjectLink | null = null;
  let editorLinkProjectId: number = 0;
  let editorLinkProjectTitle: string = '';
  let showLinkEditor = false;


  // Fold/unfold all projects — broadcast to every ProjectCard via a bumped signal.
  let allCollapsed = false;
  let collapseSignal = 0;
  function toggleFoldAll() {
    allCollapsed = !allCollapsed;
    collapseSignal++;
  }

  let cat1Label = '';
  let cat2Label = '';
  let cat3Label = '';
  let cat4Label = '';

  onMount(async () => {
    await loadAll();
    // Arriving from a template clone: focus the new project.
    const focus = get(pendingProjectFocus);
    if (focus != null) {
      pendingProjectFocus.set(null);
      focusOnProject(focus);
    }
    cat1Label = $settings.category1_label;
    cat2Label = $settings.category2_label;
    cat3Label = $settings.category3_label;
    cat4Label = $settings.category4_label;
  });

  const unsubSettings = settings.subscribe(s => {
    cat1Label = s.category1_label;
    cat2Label = s.category2_label;
    cat3Label = s.category3_label;
    cat4Label = s.category4_label;
  });
  onDestroy(unsubSettings);

  $: isDark = $settings.dark_mode;
  $: densityStyle = densityStyleFor($settings.density);

  // Template projects live only on the Templates page — hide them (and their
  // logs, via project visibility) from the whole homepage.
  $: visProjects = $projects.filter(p => !p.is_template);

  $: logTypes = $picklists.filter(v => v.picklist_type === 'log_type');
  const byLabel = (a: {label: string}, b: {label: string}) => a.label.localeCompare(b.label);
  $: cat1Vals = $picklists.filter(v => v.picklist_type === 'category_1').sort(byLabel);
  $: cat2Vals = $picklists.filter(v => v.picklist_type === 'category_2').sort(byLabel);
  $: cat3Vals = $picklists.filter(v => v.picklist_type === 'category_3').sort(byLabel);
  $: cat4Vals = $picklists.filter(v => v.picklist_type === 'category_4').sort(byLabel);

  // ── Category matching (AND logic across all selected values) ─────────────
  // Tree/visibility/matching helpers live in $lib/filters (pure + unit-tested).

  // True when no filter that narrows the visible logs is active — used to decide
  // whether projects with no matching logs should still be shown.
  $: noLogFilter = !$selCat1.length && !$selCat2.length && !$selCat3.length && !$selCat4.length
    && $selLogType === null;

  $: matchingLogs = $logs.filter(l =>
    ($selLogType === null || l.type_id === $selLogType) &&
    logMatchesSlot(l, 1, $selCat1, visProjects) &&
    logMatchesSlot(l, 2, $selCat2, visProjects) &&
    logMatchesSlot(l, 3, $selCat3, visProjects) &&
    logMatchesSlot(l, 4, $selCat4, visProjects)
  );

  // ── Project visibility ───────────────────────────────────────────────────

  $: ({ visible: visibleProjectIds, ancestorOnly: ancestorOnlyProjectIds } =
    computeVisibility(visProjects, matchingLogs, $selProject, $showClosed, noLogFilter));

  $: topLevelProjects = visProjects.filter(p => p.parent_id == null);
  $: visibleTopLevelProjects = topLevelProjects.filter(p => visibleProjectIds.has(p.id));

  function openNew(typeId: number | null = null, projectId: number | null = null) {
    editorLog = (typeId !== null || projectId !== null)
      ? ({ type_id: typeId ?? logTypes[0]?.id ?? 0, project_id: projectId ?? visProjects[0]?.id ?? 0 } as any)
      : null;
    showEditor = true;
  }

  function openEdit(e: CustomEvent<Log>) {
    editorLog = e.detail;
    showEditor = true;
  }

  function closeEditor() { showEditor = false; editorLog = null; }

  function openNewProject() { editorProject = null; showProjectEditor = true; }

  function openEditProject(e: CustomEvent<Project>) {
    editorProject = e.detail;
    showProjectEditor = true;
  }

  function closeProjectEditor() { showProjectEditor = false; editorProject = null; }

  function openNewLink(e: CustomEvent<Project>) {
    editorLink = null;
    editorLinkProjectId = e.detail.id;
    editorLinkProjectTitle = e.detail.title;
    showLinkEditor = true;
  }

  function openEditLink(e: CustomEvent<ProjectLink>) {
    editorLink = e.detail;
    const proj = visProjects.find(p => p.id === e.detail.project_id);
    editorLinkProjectId = e.detail.project_id;
    editorLinkProjectTitle = proj?.title ?? '';
    showLinkEditor = true;
  }

  function closeLinkEditor() { showLinkEditor = false; editorLink = null; }

  function openNewLogInProject(e: CustomEvent<{ project: Project; typeId: number }>) {
    openNew(e.detail.typeId, e.detail.project.id);
  }

  // After creating a project or log, reveal it: clear category filters and
  // focus the project filter on it, so it doesn't vanish behind active filters.
  function focusOnProject(projectId: number) {
    $selCat1 = []; $selCat2 = []; $selCat3 = []; $selCat4 = [];
    $selLogType = null;
    $selProject = projectId;
  }

  function onProjectCreated(e: CustomEvent<Project>) { focusOnProject(e.detail.id); }

  function onLogCreated(e: CustomEvent<Log>) { focusOnProject(Number(e.detail.project_id)); }

  function openNewSubProject(e: CustomEvent<Project>) {
    const parent = e.detail;
    editorProject = { id: 0, title: '', description: '', parent_id: parent.id, is_closed: false, is_template: false, start_date: null, end_date: null, category1_ids: [], category2_ids: [], category3_ids: [], category4_ids: [] };
    showProjectEditor = true;
  }
</script>

<div class="app" class:dark={isDark} style={densityStyle}>

  <header class="menu-bar">
    <div class="menu-left">
      <FilterPanel />
    </div>
    <nav class="menu-nav">
      <button class="btn-secondary-sm" on:click={toggleFoldAll}>{allCollapsed ? '▸ Unfold all' : '▾ Fold all'}</button>
      <button class="btn-secondary-sm" on:click={openNewProject}>+ New Project</button>
    </nav>
  </header>

    <main class="main">
      <div class="cards">
        {#each visibleTopLevelProjects as project (project.id)}
          <ProjectCard
            {project}
            subProjects={visProjects.filter(p => p.parent_id != null && Number(p.parent_id) === Number(project.id))}
            allLogs={matchingLogs}
            allLogsTotal={$logs}
            allProjects={visProjects}
            allLinks={$projectLinks}
            {visibleProjectIds}
            {ancestorOnlyProjectIds}
            showClosed={$showClosed}
            {collapseSignal}
            collapseAll={allCollapsed}
            depth={0}
            {logTypes} {cat1Vals} {cat2Vals} {cat3Vals} {cat4Vals}
            on:edit={openEdit}
            on:editProject={openEditProject}
            on:newSubProject={openNewSubProject}
            on:newLogInProject={openNewLogInProject}
            on:newLink={openNewLink}
            on:editLink={openEditLink}
          />
        {/each}

        {#if topLevelProjects.length === 0}
          <div class="empty">
            <span class="empty-icon">📋</span>
            <p>No projects yet. Click <strong>+ New Project</strong> to get started.</p>
          </div>
        {/if}
      </div>
    </main>

  {#if showEditor}
    <LogEditor
      log={editorLog}
      {logTypes} {cat1Vals} {cat2Vals} {cat3Vals} {cat4Vals}
      cat1Label={cat1Label} cat2Label={cat2Label} cat3Label={cat3Label} cat4Label={cat4Label}
      allProjects={visProjects}
      on:close={closeEditor}
      on:created={onLogCreated}
    />
  {/if}

  {#if showProjectEditor}
    <ProjectEditor
      project={editorProject}
      allProjects={visProjects}
      {cat1Vals} {cat2Vals} {cat3Vals} {cat4Vals}
      {cat1Label} {cat2Label} {cat3Label} {cat4Label}
      on:close={closeProjectEditor}
      on:created={onProjectCreated}
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

</div>

<style>
  .app {
    --accent:       #6366f1;
    --accent-muted: #a5b4fc;
    --text:         #111827;
    --text-muted:   #6b7280;
    --surface:      #ffffff;
    --surface-2:    #f3f4f6;
    --surface-3:    #e5e7eb;
    --surface-hover:#dde0e8;
    --card-bg:      #ffffff;
    --border:       #e5e7eb;
    --project-header: #ccd2dd;
    background: var(--surface-2);
    color: var(--text);
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .menu-bar {
    display: flex; align-items: center; justify-content: space-between;
    padding: 8px 20px; min-height: 52px;
    background: var(--surface); border-bottom: 1px solid var(--border);
    flex-shrink: 0; z-index: 10;
  }

  .menu-left { display: flex; align-items: center; gap: 16px; min-width: 0; flex: 1; }

  .menu-nav { display: flex; align-items: center; gap: 10px; }

  .app.dark {
    --text:         #f3f4f6;
    --text-muted:   #9ca3af;
    --surface:      #111827;
    --surface-2:    #1f2937;
    --surface-3:    #374151;
    --surface-hover:#4b5563;
    --card-bg:      #1f2937;
    --border:       #374151;
    --project-header: #374151;
  }

  .main {
    flex: 1;
    display: flex; flex-direction: column; overflow: hidden; padding: var(--sp-main-pad); min-height: 0;
  }

  .btn-new {
    background: var(--accent); color: #fff; border: none; border-radius: 10px;
    padding: 8px 16px; font-size: 14px; font-weight: 600; cursor: pointer;
    font-family: inherit; transition: opacity 0.15s, transform 0.1s;
    box-shadow: 0 2px 8px rgba(99,102,241,0.35);
  }
  .btn-new:hover { opacity: 0.9; transform: translateY(-1px); }
  .btn-new:active { transform: scale(0.97); }

  .btn-secondary-sm {
    background: none; color: var(--text-muted);
    border: 1px solid var(--border); border-radius: 10px;
    padding: 8px 14px; font-size: 14px; font-weight: 600; cursor: pointer;
    font-family: inherit; transition: background 0.15s, color 0.15s;
    text-decoration: none; display: inline-flex; align-items: center;
  }
  .btn-secondary-sm:hover { background: var(--surface-2); color: var(--text); }

  .cards {
    flex: 1; overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--sp-card-gap); padding-right: 4px;
  }

  .empty {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    gap: 12px; padding: 60px 20px; color: var(--text-muted); text-align: center;
  }
  .empty-icon { font-size: 48px; }
  .empty p { font-size: 15px; margin: 0; }

  :global(::-webkit-scrollbar) { width: 6px; }
  :global(::-webkit-scrollbar-track) { background: transparent; }
  :global(::-webkit-scrollbar-thumb) { background: var(--border); border-radius: 3px; }
</style>
