<script lang="ts">
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import CategoryFilter from '$lib/components/CategoryFilter.svelte';
  import LogCard from '$lib/components/LogCard.svelte';
  import ProjectCard from '$lib/components/ProjectCard.svelte';
  import LogEditor from '$lib/components/LogEditor.svelte';
  import ProjectEditor from '$lib/components/ProjectEditor.svelte';
  import SettingsPanel from '$lib/components/SettingsPanel.svelte';
  import { settings, picklists, projects, logs, loadAll, saveSettings } from '$lib/store';
  import type { Log, Project } from '$lib/types';

  const DENSITY_VARS: Record<string, Record<string, string>> = {
    compact: {
      '--sp-main-pad': '10px',
      '--sp-card-gap': '6px',
      '--sp-sidebar-pad': '8px',
      '--sp-topcat-pad': '6px 10px',
      '--sp-card-pad': '8px 10px',
      '--sp-card-gap-inner': '4px',
      '--sp-panel-pad': '14px',
      '--sp-panel-gap': '12px',
      '--sp-field-gap': '4px',
    },
    normal: {
      '--sp-main-pad': '24px',
      '--sp-card-gap': '12px',
      '--sp-sidebar-pad': '16px',
      '--sp-topcat-pad': '10px 16px',
      '--sp-card-pad': '14px 16px',
      '--sp-card-gap-inner': '8px',
      '--sp-panel-pad': '24px',
      '--sp-panel-gap': '16px',
      '--sp-field-gap': '5px',
    },
    comfortable: {
      '--sp-main-pad': '32px',
      '--sp-card-gap': '18px',
      '--sp-sidebar-pad': '22px',
      '--sp-topcat-pad': '16px 22px',
      '--sp-card-pad': '20px 22px',
      '--sp-card-gap-inner': '12px',
      '--sp-panel-pad': '32px',
      '--sp-panel-gap': '22px',
      '--sp-field-gap': '8px',
    },
  };

  let editorLog: Log | null = null;
  let showEditor = false;
  let showSettings = false;
  let editorProject: Project | null = null;
  let showProjectEditor = false;

  // Filter state
  let showClosed = false;
  let selCat1: number[] = [];
  let selCat2: number[] = [];
  let selCat3: number[] = [];
  let selCat4: number[] = [];
  let selProject: number | null = null;

  let cat1Label = '';
  let cat2Label = '';
  let cat3Label = '';
  let cat4Label = '';

  onMount(async () => {
    await loadAll();
    cat1Label = $settings.category1_label;
    cat2Label = $settings.category2_label;
    cat3Label = $settings.category3_label;
    cat4Label = $settings.category4_label;
  });

  settings.subscribe(s => {
    cat1Label = s.category1_label;
    cat2Label = s.category2_label;
    cat3Label = s.category3_label;
    cat4Label = s.category4_label;
  });

  async function persistLabelChange() {
    const s = get(settings);
    await saveSettings({ ...s, category1_label: cat1Label, category2_label: cat2Label, category3_label: cat3Label, category4_label: cat4Label });
  }

  $: isDark = $settings.dark_mode;
  $: densityStyle = Object.entries(DENSITY_VARS[$settings.density] ?? DENSITY_VARS.normal)
    .map(([k, v]) => `${k}:${v}`).join(';');

  $: logTypes = $picklists.filter(v => v.picklist_type === 'log_type');
  const byLabel = (a: {label: string}, b: {label: string}) => a.label.localeCompare(b.label);
  $: cat1Vals = $picklists.filter(v => v.picklist_type === 'category_1').sort(byLabel);
  $: cat2Vals = $picklists.filter(v => v.picklist_type === 'category_2').sort(byLabel);
  $: cat3Vals = $picklists.filter(v => v.picklist_type === 'category_3').sort(byLabel);
  $: cat4Vals = $picklists.filter(v => v.picklist_type === 'category_4').sort(byLabel);

  $: filtersActive = selCat1.length > 0 || selCat2.length > 0 || selCat3.length > 0 || selCat4.length > 0 || selProject !== null;
  $: catOrCloseFilterActive = selCat1.length > 0 || selCat2.length > 0 || selCat3.length > 0 || selCat4.length > 0 || showClosed;
  $: anyFilterActive = filtersActive || showClosed;

  function clearAllFilters() {
    selCat1 = []; selCat2 = []; selCat3 = []; selCat4 = [];
    selProject = null;
    showClosed = false;
  }

  function getSubtreeIds(projectId: number): Set<number> {
    const ids = new Set<number>([projectId]);
    $projects.filter(p => p.parent_id !== null && Number(p.parent_id) === projectId)
      .forEach(child => getSubtreeIds(child.id).forEach(id => ids.add(id)));
    return ids;
  }

  function getRootAncestorId(projectId: number): number {
    const p = $projects.find(p => p.id === projectId);
    if (!p || p.parent_id == null) return projectId;
    return getRootAncestorId(Number(p.parent_id));
  }

  $: subtreeIds = selProject !== null ? getSubtreeIds(selProject) : null;

  function ancestorCatIds(projectId: number, slot: 1|2|3|4): number[] {
    const p = $projects.find(p => p.id === projectId);
    if (!p) return [];
    const own = p[`category${slot}_ids`] as number[];
    if (p.parent_id != null) return [...own, ...ancestorCatIds(Number(p.parent_id), slot)];
    return own;
  }

  function logPassesCat(l: import('$lib/types').Log, slot: 1|2|3|4, sel: number[]): boolean {
    if (!sel.length) return true;
    const logIds = l[`category${slot}_ids`];
    const projIds = l.project_id != null ? ancestorCatIds(Number(l.project_id), slot) : [];
    return sel.every(id => logIds.includes(id) || projIds.includes(id));
  }

  $: filtered = $logs.filter(l => {
    if (!showClosed && l.is_closed) return false;
    if (!logPassesCat(l, 1, selCat1)) return false;
    if (!logPassesCat(l, 2, selCat2)) return false;
    if (!logPassesCat(l, 3, selCat3)) return false;
    if (!logPassesCat(l, 4, selCat4)) return false;
    if (subtreeIds !== null && (l.project_id == null || !subtreeIds.has(Number(l.project_id)))) return false;
    return true;
  });

  $: topLevelProjects = $projects.filter(p => p.parent_id == null);
  $: visibleTopLevelProjects = selProject !== null
    ? topLevelProjects.filter(p => p.id === getRootAncestorId(selProject!))
    : topLevelProjects;
  $: unassignedLogs = selProject !== null ? [] : filtered.filter(l => l.project_id == null);

  // Build flat project list for dropdown (indented to show hierarchy)
  function buildProjectOptions(projects: Project[], parentId: number | null = null, depth = 0): {id: number, label: string}[] {
    return projects
      .filter(p => (p.parent_id == null) === (parentId == null) && (parentId == null || Number(p.parent_id) === parentId))
      .sort((a, b) => a.title.localeCompare(b.title))
      .flatMap(p => [
        { id: p.id, label: '  '.repeat(depth) + p.title },
        ...buildProjectOptions(projects, p.id, depth + 1)
      ]);
  }
  $: projectOptions = buildProjectOptions($projects);

  function projectHasAnyLogs(projectId: number): boolean {
    if (filtered.some(l => Number(l.project_id) === Number(projectId))) return true;
    return $projects
      .filter(p => p.parent_id != null && Number(p.parent_id) === Number(projectId))
      .some(sub => projectHasAnyLogs(sub.id));
  }

  function subProjectsOf(projectId: number): Project[] {
    return $projects.filter(p => p.parent_id != null && Number(p.parent_id) === Number(projectId));
  }

  function openNew(typeId: number | null = null, projectId: number | null = null) {
    editorLog = (typeId !== null || projectId !== null)
      ? ({ type_id: typeId ?? logTypes[0]?.id ?? 0, project_id: projectId } as any)
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

  function openNewLogInProject(e: CustomEvent<Project>) {
    openNew(null, e.detail.id);
  }
</script>

<div class="app" class:dark={isDark} style={densityStyle}>

  <header class="menu-bar">
    <div class="menu-left">
      <svg width="36" height="40" viewBox="0 0 36 40" aria-label="Note Ledger">
        <rect x="0" y="1" width="6" height="38" rx="1.5" fill="#4f46e5"/>
        <rect x="6" y="1" width="27" height="38" rx="1.5" fill="#6366f1"/>
        <rect x="32" y="3" width="4" height="34" rx="0.5" fill="#e0e7ff"/>
        <line x1="32" y1="9"  x2="36" y2="9"  stroke="#a5b4fc" stroke-width="0.6"/>
        <line x1="32" y1="15" x2="36" y2="15" stroke="#a5b4fc" stroke-width="0.6"/>
        <line x1="32" y1="21" x2="36" y2="21" stroke="#a5b4fc" stroke-width="0.6"/>
        <line x1="32" y1="27" x2="36" y2="27" stroke="#a5b4fc" stroke-width="0.6"/>
        <line x1="32" y1="33" x2="36" y2="33" stroke="#a5b4fc" stroke-width="0.6"/>
        <text
          x="19" y="20"
          text-anchor="middle" dominant-baseline="middle"
          transform="rotate(-90, 19, 20)"
          font-family="-apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif"
          font-size="6.5" font-weight="800" fill="#fff" opacity="0.95" letter-spacing="0.3">NLedger</text>
      </svg>
      <label class="toggle-wrap" title="Show closed logs">
        <span class="toggle-switch" class:on={showClosed} on:click={() => showClosed = !showClosed} role="switch" aria-checked={showClosed} tabindex="0" on:keydown={e => e.key === ' ' && (showClosed = !showClosed)}>
          <span class="toggle-thumb"></span>
        </span>
        <span class="toggle-label">Show closed</span>
      </label>
      {#if $projects.length > 0}
        <select class="project-filter" value={selProject === null ? '' : String(selProject)} on:change={e => { const v = (e.target as HTMLSelectElement).value; selProject = v === '' ? null : Number(v); }}>
          <option value="">All projects</option>
          {#each projectOptions as opt}
            <option value={String(opt.id)}>{opt.label}</option>
          {/each}
        </select>
      {/if}
      <button class="btn-clear-filters" on:click={clearAllFilters}>✕ Clear filters</button>
    </div>
    <nav class="menu-nav">
      <button class="btn-secondary-sm" on:click={openNewProject}>+ New Project</button>
      {#if logTypes.length > 0}
        {#each logTypes as lt}
          <button class="btn-new" on:click={() => openNew(lt.id)}>+ {lt.label}</button>
        {/each}
      {:else}
        <button class="btn-new" on:click={() => openNew()}>+ New Log</button>
      {/if}
      <button class="theme-toggle" on:click={() => showSettings = true} title="Settings">⚙️</button>
    </nav>
  </header>

  <div class="layout">
    <div class="top-cats">
      <div class="top-cat-col">
        <CategoryFilter catType="category_3" bind:label={cat3Label} values={cat3Vals} bind:selected={selCat3} layout="horizontal" on:labelChange={persistLabelChange} />
      </div>
      <div class="top-cat-separator"></div>
      <div class="top-cat-col">
        <CategoryFilter catType="category_4" bind:label={cat4Label} values={cat4Vals} bind:selected={selCat4} layout="horizontal" on:labelChange={persistLabelChange} />
      </div>
    </div>

    <aside class="sidebar">
      <div class="sidebar-cat">
        <CategoryFilter catType="category_1" bind:label={cat1Label} values={cat1Vals} bind:selected={selCat1} layout="vertical" on:labelChange={persistLabelChange} />
      </div>
      <div class="sidebar-separator"></div>
      <div class="sidebar-cat">
        <CategoryFilter catType="category_2" bind:label={cat2Label} values={cat2Vals} bind:selected={selCat2} layout="vertical" on:labelChange={persistLabelChange} />
      </div>
    </aside>

    <main class="main">
      <div class="cards">
        {#each visibleTopLevelProjects as project (project.id)}
          {#if selProject !== null || !anyFilterActive || projectHasAnyLogs(project.id)}
            <ProjectCard
              {project}
              subProjects={$projects.filter(p => p.parent_id != null && Number(p.parent_id) === Number(project.id))}
              allLogs={filtered}
              allLogsTotal={$logs}
              allProjects={$projects}
              filtersActive={anyFilterActive}
              showAllSubProjects={selProject !== null && !catOrCloseFilterActive}
              {logTypes} {cat1Vals} {cat2Vals} {cat3Vals} {cat4Vals}
              on:edit={openEdit}
              on:editProject={openEditProject}
              on:newLogInProject={openNewLogInProject}
            />
          {/if}
        {/each}

        {#each unassignedLogs as log (log.id)}
          <LogCard {log} {logTypes} {cat1Vals} {cat2Vals} {cat3Vals} {cat4Vals} on:edit={openEdit} />
        {/each}


        {#if topLevelProjects.length === 0 && unassignedLogs.length === 0}
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
      allProjects={$projects}
      on:close={closeEditor}
    />
  {/if}

  {#if showProjectEditor}
    <ProjectEditor
      project={editorProject}
      allProjects={$projects}
      {cat1Vals} {cat2Vals} {cat3Vals} {cat4Vals}
      {cat1Label} {cat2Label} {cat3Label} {cat4Label}
      on:close={closeProjectEditor}
    />
  {/if}

  {#if showSettings}
    <SettingsPanel on:close={() => showSettings = false} />
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
    height: 100vh;
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

  .theme-toggle {
    background: none; border: 1px solid var(--border); border-radius: 8px;
    width: 34px; height: 34px; cursor: pointer; font-size: 16px;
    display: flex; align-items: center; justify-content: center; transition: background 0.15s;
  }
  .theme-toggle:hover { background: var(--surface-2); }

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
    grid-template-rows: auto 1fr;
    flex: 1; overflow: hidden; min-height: 0;
  }

  .sidebar {
    grid-column: 1; grid-row: 1 / 3;
    background: var(--surface); border-right: 1px solid var(--border);
    display: flex; flex-direction: column; overflow: hidden; min-height: 0;
  }
  .sidebar-cat { flex: 1; overflow-y: auto; padding: var(--sp-sidebar-pad); min-height: 0; }
  .sidebar-separator { height: 1px; background: var(--border); flex-shrink: 0; margin: 0 12px; }

  .top-cats {
    grid-column: 2; grid-row: 1;
    display: flex; flex-direction: row; align-items: stretch;
    background: var(--surface); border-bottom: 1px solid var(--border);
  }
  .top-cat-col { flex: 1; padding: var(--sp-topcat-pad); min-width: 0; }
  .top-cat-separator { width: 1px; background: var(--border); flex-shrink: 0; }

  .main {
    grid-column: 2; grid-row: 2;
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

  .btn-clear-filters {
    background: none; color: #ef4444;
    border: 1px solid #ef4444; border-radius: 8px;
    padding: 6px 12px; font-size: 13px; font-weight: 600;
    cursor: pointer; font-family: inherit; transition: background 0.15s;
  }
  .btn-clear-filters:hover { background: rgba(239,68,68,0.1); }

  .project-filter {
    background: var(--surface-2); color: var(--text);
    border: 1px solid var(--border); border-radius: 8px;
    padding: 6px 10px; font-size: 13px; font-family: inherit;
    cursor: pointer; outline: none; max-width: 160px;
  }
  .project-filter:focus { border-color: var(--accent); }

  .btn-secondary-sm {
    background: none; color: var(--text-muted);
    border: 1px solid var(--border); border-radius: 10px;
    padding: 8px 14px; font-size: 14px; font-weight: 600; cursor: pointer;
    font-family: inherit; transition: background 0.15s, color 0.15s;
  }
  .btn-secondary-sm:hover { background: var(--surface-2); color: var(--text); }

  .cards {
    flex: 1; overflow-y: auto;
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    grid-auto-rows: min-content;
    align-content: start;
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
