<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import CategoryFilter from '$lib/components/CategoryFilter.svelte';
  import ProjectCard from '$lib/components/ProjectCard.svelte';
  import LogEditor from '$lib/components/LogEditor.svelte';
  import ProjectEditor from '$lib/components/ProjectEditor.svelte';
  import LinkEditor from '$lib/components/LinkEditor.svelte';
  import { settings, picklists, projects, logs, projectLinks, loadAll, saveSettings, pendingProjectFocus } from '$lib/store';
  import type { Log, Project, ProjectLink } from '$lib/types';
  import { sortedProjectOptions } from '$lib/types';
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

  // Filter state
  let showClosed = false;
  let selCat1: number[] = [];
  let selCat2: number[] = [];
  let selCat3: number[] = [];
  let selCat4: number[] = [];
  let selProject: number | null = null;
  let selLogType: number | null = null;

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

  async function persistLabelChange() {
    const s = get(settings);
    await saveSettings({ ...s, category1_label: cat1Label, category2_label: cat2Label, category3_label: cat3Label, category4_label: cat4Label });
  }

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

  function clearAllFilters() {
    selCat1 = []; selCat2 = []; selCat3 = []; selCat4 = [];
    selProject = null;
    selLogType = null;
    showClosed = false;
  }

  // ── Category matching (AND logic across all selected values) ─────────────
  // Tree/visibility/matching helpers live in $lib/filters (pure + unit-tested).

  // True when no filter that narrows the visible logs is active — used to decide
  // whether projects with no matching logs should still be shown.
  $: noLogFilter = !selCat1.length && !selCat2.length && !selCat3.length && !selCat4.length
    && selLogType === null;

  $: matchingLogs = $logs.filter(l =>
    (selLogType === null || l.type_id === selLogType) &&
    logMatchesSlot(l, 1, selCat1, visProjects) &&
    logMatchesSlot(l, 2, selCat2, visProjects) &&
    logMatchesSlot(l, 3, selCat3, visProjects) &&
    logMatchesSlot(l, 4, selCat4, visProjects)
  );

  // ── Project visibility ───────────────────────────────────────────────────

  $: ({ visible: visibleProjectIds, ancestorOnly: ancestorOnlyProjectIds } =
    computeVisibility(visProjects, matchingLogs, selProject, showClosed, noLogFilter));

  $: topLevelProjects = visProjects.filter(p => p.parent_id == null);
  $: visibleTopLevelProjects = topLevelProjects.filter(p => visibleProjectIds.has(p.id));

  $: projectOptions = sortedProjectOptions(visProjects);

  // ── Project lookup ────────────────────────────────────────────────────────
  let lookupSearch = '';
  let lookupOpen = false;
  let lookupInput: HTMLInputElement;

  $: selectedProjectLabel = selProject != null
    ? (visProjects.find(p => p.id === selProject)?.title ?? '')
    : '';

  $: filteredProjectOptions = (() => {
    const tokens = lookupSearch.trim().toLowerCase().split(/\s+/).filter(Boolean);
    return projectOptions.filter(opt => {
      const title = opt.label.trim().toLowerCase();
      return tokens.length === 0 || tokens.every(t => title.includes(t));
    });
  })();

  function selectProject(id: number | null) {
    selProject = id;
    lookupSearch = '';
    lookupOpen = false;
    lookupInput?.blur();
  }

  function onLookupBlur() {
    // small delay so click on option fires first
    setTimeout(() => { lookupOpen = false; lookupSearch = ''; }, 150);
  }

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
    selCat1 = []; selCat2 = []; selCat3 = []; selCat4 = [];
    selLogType = null;
    selProject = projectId;
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
      <label class="toggle-wrap" title="Show closed projects">
        <span class="toggle-switch" class:on={showClosed} on:click={() => showClosed = !showClosed} role="switch" aria-checked={showClosed} tabindex="0" on:keydown={e => e.key === ' ' && (showClosed = !showClosed)}>
          <span class="toggle-thumb"></span>
        </span>
        <span class="toggle-label">Show closed</span>
      </label>
      {#if visProjects.length > 0}
        <div class="project-lookup" class:active={lookupOpen}>
          <input
            bind:this={lookupInput}
            class="project-lookup-input"
            type="text"
            placeholder={selProject != null ? selectedProjectLabel : 'Filter project…'}
            bind:value={lookupSearch}
            on:focus={() => { lookupOpen = true; lookupSearch = ''; selProject = null; }}
            on:blur={onLookupBlur}
          />
          {#if lookupOpen}
            <div class="lookup-dropdown">
              {#each filteredProjectOptions as opt}
                {@const proj = visProjects.find(p => p.id === opt.id)}
                <button
                  class="lookup-option"
                  class:is-closed={proj?.is_closed}
                  on:mousedown|preventDefault={() => selectProject(opt.id)}
                >
                  <span class="lookup-label">{opt.label}</span>
                  {#if proj?.is_closed}
                    <span class="lookup-closed-pill">Closed</span>
                  {/if}
                </button>
              {/each}
              {#if filteredProjectOptions.length === 0}
                <div class="lookup-empty">No match</div>
              {/if}
            </div>
          {/if}
        </div>
      {/if}
      {#if logTypes.length > 0}
        <select class="logtype-filter" class:active={selLogType !== null} bind:value={selLogType} title="Filter by log type">
          <option value={null}>All types</option>
          {#each logTypes as t (t.id)}
            <option value={t.id}>{t.label}</option>
          {/each}
        </select>
      {/if}
      <button class="btn-clear-filters" on:click={clearAllFilters}>✕ Clear filters</button>
    </div>
    <nav class="menu-nav">
      <button class="btn-secondary-sm" on:click={toggleFoldAll}>{allCollapsed ? '▸ Unfold all' : '▾ Fold all'}</button>
      <button class="btn-secondary-sm" on:click={openNewProject}>+ New Project</button>
    </nav>
  </header>

  <div class="layout">
    <aside class="sidebar">
      <div class="sidebar-cat">
        <CategoryFilter catType="category_1" bind:label={cat1Label} values={cat1Vals} bind:selected={selCat1} layout="vertical" on:labelChange={persistLabelChange} />
      </div>
      <div class="sidebar-separator"></div>
      <div class="sidebar-cat">
        <CategoryFilter catType="category_2" bind:label={cat2Label} values={cat2Vals} bind:selected={selCat2} layout="vertical" on:labelChange={persistLabelChange} />
      </div>
      <div class="sidebar-separator"></div>
      <div class="sidebar-cat">
        <CategoryFilter catType="category_3" bind:label={cat3Label} values={cat3Vals} bind:selected={selCat3} layout="vertical" on:labelChange={persistLabelChange} />
      </div>
      <div class="sidebar-separator"></div>
      <div class="sidebar-cat">
        <CategoryFilter catType="category_4" bind:label={cat4Label} values={cat4Vals} bind:selected={selCat4} layout="vertical" on:labelChange={persistLabelChange} />
      </div>
    </aside>

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
            {showClosed}
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
            <p>No logs yet. Click <strong>+ New Log</strong> to get started.</p>
          </div>
        {/if}
      </div>
    </main>
  </div>

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
    padding: 0 20px; height: 52px;
    background: var(--surface); border-bottom: 1px solid var(--border);
    flex-shrink: 0; z-index: 10;
  }

  .menu-left { display: flex; align-items: center; gap: 16px; }

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
  }

  .layout {
    display: grid;
    grid-template-columns: 230px 1fr;
    grid-template-rows: 1fr;
    flex: 1; overflow: hidden; min-height: 0;
  }

  .sidebar {
    grid-column: 1; grid-row: 1;
    background: var(--surface); border-right: 1px solid var(--border);
    display: flex; flex-direction: column; overflow: hidden; min-height: 0;
  }
  .sidebar-cat { flex: 1; overflow-y: auto; padding: var(--sp-sidebar-pad); min-height: 0; }
  .sidebar-separator { height: 1px; background: var(--border); flex-shrink: 0; margin: 0 12px; }

  .main {
    grid-column: 2; grid-row: 1;
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

  .toggle-wrap {
    display: flex; align-items: center; gap: 7px; cursor: pointer; user-select: none;
  }
  .toggle-label {
    font-size: 13px; font-weight: 600; color: var(--text-muted);
  }
  .toggle-switch {
    width: 36px; height: 20px; border-radius: 10px;
    background: var(--surface-3); border: 1px solid var(--border);
    position: relative; cursor: pointer;
    transition: background 0.2s, border-color 0.2s;
    display: flex; align-items: center;
  }
  .toggle-switch.on { background: var(--accent); border-color: var(--accent); }
  .toggle-thumb {
    width: 14px; height: 14px; border-radius: 50%;
    background: #fff; position: absolute; left: 2px;
    transition: left 0.2s; box-shadow: 0 1px 3px rgba(0,0,0,0.2);
  }
  .toggle-switch.on .toggle-thumb { left: 18px; }

  .logtype-filter {
    background: var(--surface-2); color: var(--text);
    border: 1px solid var(--border); border-radius: 8px;
    padding: 6px 10px; font-size: 13px; font-family: inherit;
    outline: none; cursor: pointer;
    transition: border-color 0.15s;
  }
  .logtype-filter:focus { border-color: var(--accent); }
  .logtype-filter.active { border-color: var(--accent); color: var(--accent); font-weight: 600; }

  .btn-clear-filters {
    background: none; color: #ef4444;
    border: 1px solid #ef4444; border-radius: 8px;
    padding: 6px 12px; font-size: 13px; font-weight: 600;
    cursor: pointer; font-family: inherit; transition: background 0.15s;
  }
  .btn-clear-filters:hover { background: rgba(239,68,68,0.1); }

  .project-lookup {
    position: relative;
    display: flex; align-items: center;
  }
  .project-lookup-input {
    background: var(--surface-2); color: var(--text);
    border: 1px solid var(--border); border-radius: 8px;
    padding: 6px 28px 6px 10px; font-size: 13px; font-family: inherit;
    outline: none; width: 180px;
    transition: border-color 0.15s;
  }
  .project-lookup.active .project-lookup-input,
  .project-lookup-input:focus { border-color: var(--accent); }
  .lookup-clear {
    position: absolute; right: 6px;
    background: none; border: none; cursor: pointer;
    color: var(--text-muted); font-size: 11px; padding: 2px;
    line-height: 1;
  }
  .lookup-clear:hover { color: var(--text); }
  .lookup-dropdown {
    position: absolute; top: calc(100% + 4px); left: 0;
    min-width: 220px; max-height: 280px; overflow-y: auto;
    background: var(--surface); border: 1px solid var(--border);
    border-radius: 8px; box-shadow: 0 4px 16px rgba(0,0,0,0.15);
    z-index: 300;
    display: flex; flex-direction: column;
  }
  .lookup-option {
    display: flex; align-items: center; justify-content: space-between; gap: 8px;
    width: 100%; text-align: left;
    background: none; border: none; cursor: pointer;
    padding: 7px 12px; font-size: 13px; font-family: inherit;
    color: var(--text); white-space: pre;
    transition: background 0.1s;
  }
  .lookup-option:hover { background: var(--surface-2); }
  .lookup-option.is-closed { opacity: 0.6; }
  .lookup-label { flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; }
  .lookup-closed-pill {
    font-size: 10px; font-weight: 600; text-transform: uppercase;
    background: var(--border); color: var(--text-muted);
    border-radius: 999px; padding: 1px 6px; flex-shrink: 0;
    white-space: normal;
  }
  .lookup-empty { padding: 8px 12px; font-size: 12px; color: var(--text-muted); }

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
