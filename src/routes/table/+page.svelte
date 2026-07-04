<script lang="ts">
  import { onMount } from 'svelte';
  import { get } from 'svelte/store';
  import { logs, projects, picklists, settings, loadAll, saveSettings, showClosed, selCat1, selCat2, selCat3, selCat4, selProject, selLogType } from '$lib/store';
  import { deadlineColor, contrastText, handleLinkClick, openSinceLabel, sortedProjectOptions } from '$lib/types';
  import type { Log, Project, PicklistValue } from '$lib/types';
  import Badge from '$lib/components/Badge.svelte';
  import CategoryFilter from '$lib/components/CategoryFilter.svelte';
  import LogEditor from '$lib/components/LogEditor.svelte';
  import ProjectEditor from '$lib/components/ProjectEditor.svelte';
  import { logMatchesSlot, getDescendantIds } from '$lib/filters';
  import { densityStyle } from '$lib/density';

  onMount(() => loadAll());


  function clearAllFilters() {
    $selCat1 = []; $selCat2 = []; $selCat3 = []; $selCat4 = [];
    $selProject = null;
    $selLogType = null;
    $showClosed = false;
  }

  let cat1Label = '';
  let cat2Label = '';
  let cat3Label = '';
  let cat4Label = '';
  $: { cat1Label = $settings.category1_label; cat2Label = $settings.category2_label; cat3Label = $settings.category3_label; cat4Label = $settings.category4_label; }

  async function persistLabelChange() {
    const s = get(settings);
    await saveSettings({ ...s, category1_label: cat1Label, category2_label: cat2Label, category3_label: cat3Label, category4_label: cat4Label });
  }

  $: pageDensityStyle = densityStyle($settings.density);

  // ── Project lookup (same behavior as the homepage) ────────────────────────
  let lookupSearch = '';
  let lookupOpen = false;
  let lookupInput: HTMLInputElement;

  $: projectOptions = sortedProjectOptions(nonTemplateProjects);
  $: selectedProjectLabel = $selProject != null
    ? (nonTemplateProjects.find(p => p.id === $selProject)?.title ?? '')
    : '';
  $: filteredProjectOptions = (() => {
    const tokens = lookupSearch.trim().toLowerCase().split(/\s+/).filter(Boolean);
    return projectOptions.filter(opt => {
      const title = opt.label.trim().toLowerCase();
      return tokens.length === 0 || tokens.every(t => title.includes(t));
    });
  })();

  function selectProject(id: number | null) {
    $selProject = id;
    lookupSearch = '';
    lookupOpen = false;
    lookupInput?.blur();
  }

  function onLookupBlur() {
    setTimeout(() => { lookupOpen = false; lookupSearch = ''; }, 150);
  }

  let editorLog: Log | null = null;
  let showLogEditor = false;
  let editorProject: Project | null = null;
  let showProjectEditor = false;

  function openLogEdit(log: Log) {
    editorLog = log;
    showLogEditor = true;
  }

  function openProjectEdit(log: Log) {
    const p = getProject(log, $projects);
    if (!p) return;
    editorProject = p;
    showProjectEditor = true;
  }

  $: logTypes = $picklists.filter(v => v.picklist_type === 'log_type');
  const byLabel = (a: PicklistValue, b: PicklistValue) => a.label.localeCompare(b.label);
  $: cat1Vals = $picklists.filter(v => v.picklist_type === 'category_1').sort(byLabel);
  $: cat2Vals = $picklists.filter(v => v.picklist_type === 'category_2').sort(byLabel);
  $: cat3Vals = $picklists.filter(v => v.picklist_type === 'category_3').sort(byLabel);
  $: cat4Vals = $picklists.filter(v => v.picklist_type === 'category_4').sort(byLabel);

  // These helpers take the store snapshots as arguments (rather than reading
  // $projects/$picklists internally) so template calls re-run on store updates.
  function getProject(log: Log, allProjects: Project[]): Project | undefined {
    return allProjects.find(p => p.id === log.project_id);
  }

  function getProjectPath(log: Log, allProjects: Project[]): string {
    const p = getProject(log, allProjects);
    if (!p) return '—';
    const parts: string[] = [p.title];
    let cur = p;
    while (cur.parent_id) {
      const parent = allProjects.find(x => x.id === cur.parent_id);
      if (!parent) break;
      parts.unshift(parent.title);
      cur = parent;
    }
    return parts.join(' › ');
  }

  // Collect all category badges for a project (own only, not inherited)
  function getProjectBadges(log: Log, allProjects: Project[], allPicklists: PicklistValue[]): { label: string; catType: string }[] {
    const p = getProject(log, allProjects);
    if (!p) return [];
    const badges: { label: string; catType: string }[] = [];
    const slots: [keyof typeof p, string][] = [
      ['category1_ids', 'category_1'],
      ['category2_ids', 'category_2'],
      ['category3_ids', 'category_3'],
      ['category4_ids', 'category_4'],
    ];
    for (const [key, catType] of slots) {
      const ids = p[key] as number[];
      for (const id of ids) {
        const val = allPicklists.find(v => v.id === id);
        if (val) badges.push({ label: val.label, catType });
      }
    }
    return badges;
  }

  function getLogTypeLabel(log: Log, allPicklists: PicklistValue[]): string {
    return allPicklists.find(v => v.id === log.type_id && v.picklist_type === 'log_type')?.label ?? '';
  }

  function getLogBadges(log: Log, allPicklists: PicklistValue[]): { label: string; catType: string }[] {
    const badges: { label: string; catType: string }[] = [];
    const slots: [number[], string][] = [
      [log.category1_ids, 'category_1'],
      [log.category2_ids, 'category_2'],
      [log.category3_ids, 'category_3'],
      [log.category4_ids, 'category_4'],
    ];
    for (const [ids, catType] of slots) {
      for (const id of ids) {
        const val = allPicklists.find(v => v.id === id);
        if (val) badges.push({ label: val.label, catType });
      }
    }
    return badges;
  }

  // Template projects (and their logs) belong to the Templates page only.
  $: nonTemplateProjects = $projects.filter(p => !p.is_template);
  $: templateProjectIds = new Set($projects.filter(p => p.is_template).map(p => p.id));

  // Project filter narrows to the selected project's subtree.
  $: selProjectIds = $selProject != null ? getDescendantIds(nonTemplateProjects, $selProject) : null;

  // Same filter semantics as the homepage; "Show closed" here reveals closed
  // logs (the page is log-centric, hidden-by-default like closed projects).
  $: matchingLogs = $logs.filter(l =>
    !templateProjectIds.has(Number(l.project_id)) &&
    ($showClosed || !l.is_closed) &&
    (selProjectIds === null || selProjectIds.has(Number(l.project_id))) &&
    ($selLogType === null || l.type_id === $selLogType) &&
    logMatchesSlot(l, 1, $selCat1, nonTemplateProjects) &&
    logMatchesSlot(l, 2, $selCat2, nonTemplateProjects) &&
    logMatchesSlot(l, 3, $selCat3, nonTemplateProjects) &&
    logMatchesSlot(l, 4, $selCat4, nonTemplateProjects)
  );

  // Open logs sorted by due date ASC (no date last), closed logs at the end.
  $: sortedLogs = [...matchingLogs].sort((a, b) => {
    if (a.is_closed !== b.is_closed) return a.is_closed ? 1 : -1;
    if (!a.due_date && !b.due_date) return 0;
    if (!a.due_date) return 1;
    if (!b.due_date) return -1;
    return a.due_date.localeCompare(b.due_date);
  });
</script>

<div class="page" style={pageDensityStyle}>
  <header class="page-header">
    <h1>Table view</h1>
    <span class="count">{sortedLogs.length} log{sortedLogs.length !== 1 ? 's' : ''}</span>
    <label class="toggle-wrap" title="Show closed logs">
      <span class="toggle-switch" class:on={$showClosed} on:click={() => $showClosed = !$showClosed} role="switch" aria-checked={$showClosed} tabindex="0" on:keydown={e => e.key === ' ' && ($showClosed = !$showClosed)}>
        <span class="toggle-thumb"></span>
      </span>
      <span class="toggle-label">Show closed</span>
    </label>
    {#if nonTemplateProjects.length > 0}
      <div class="project-lookup" class:active={lookupOpen}>
        <input
          bind:this={lookupInput}
          class="project-lookup-input"
          type="text"
          placeholder={$selProject != null ? selectedProjectLabel : 'Filter project…'}
          bind:value={lookupSearch}
          on:focus={() => { lookupOpen = true; lookupSearch = ''; $selProject = null; }}
          on:blur={onLookupBlur}
          on:keydown={e => { if (e.key === 'Enter' && filteredProjectOptions.length > 0) selectProject(filteredProjectOptions[0].id); if (e.key === 'Escape') { lookupOpen = false; lookupSearch = ''; lookupInput?.blur(); } }}
        />
        {#if lookupOpen}
          <div class="lookup-dropdown">
            {#each filteredProjectOptions as opt}
              {@const proj = nonTemplateProjects.find(p => p.id === opt.id)}
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
      <select class="logtype-filter" class:active={$selLogType !== null} bind:value={$selLogType} title="Filter by log type">
        <option value={null}>All types</option>
        {#each logTypes as t (t.id)}
          <option value={t.id}>{t.label}</option>
        {/each}
      </select>
    {/if}
    <button class="btn-clear-filters" on:click={clearAllFilters}>✕ Clear filters</button>
  </header>

  <div class="layout">
    <aside class="sidebar">
      <div class="sidebar-cat">
        <CategoryFilter catType="category_1" bind:label={cat1Label} values={cat1Vals} bind:selected={$selCat1} layout="vertical" on:labelChange={persistLabelChange} />
      </div>
      <div class="sidebar-separator"></div>
      <div class="sidebar-cat">
        <CategoryFilter catType="category_2" bind:label={cat2Label} values={cat2Vals} bind:selected={$selCat2} layout="vertical" on:labelChange={persistLabelChange} />
      </div>
      <div class="sidebar-separator"></div>
      <div class="sidebar-cat">
        <CategoryFilter catType="category_3" bind:label={cat3Label} values={cat3Vals} bind:selected={$selCat3} layout="vertical" on:labelChange={persistLabelChange} />
      </div>
      <div class="sidebar-separator"></div>
      <div class="sidebar-cat">
        <CategoryFilter catType="category_4" bind:label={cat4Label} values={cat4Vals} bind:selected={$selCat4} layout="vertical" on:labelChange={persistLabelChange} />
      </div>
    </aside>

  <div class="table-wrap">
    {#if sortedLogs.length === 0}
      <p class="empty">No logs matching the filters.</p>
    {:else}
      <div class="table-outer"><table class="log-table">
        <thead>
          <tr>
            <th class="col-project">Project</th>
            <th class="col-title">Log</th>
            <th class="col-deadline">Deadline</th>
            <th class="col-desc">Description</th>
          </tr>
        </thead>
        <tbody>
          {#each sortedLogs as log (log.id)}
            {@const dl = log.due_date && !log.is_closed ? deadlineColor(log.due_date) : null}
            {@const dlText = dl ? contrastText(dl) : '#fff'}
            {@const projBadges = getProjectBadges(log, $projects, $picklists)}
            {@const logBadges = getLogBadges(log, $picklists)}
            {@const typeMeta = [getLogTypeLabel(log, $picklists), openSinceLabel(log)].filter(Boolean).join(' · ')}
            <tr
              class="log-row"
              class:log-closed={log.is_closed}
              on:click={() => openLogEdit(log)}
              role="button" tabindex="0"
              on:keydown={e => e.key === 'Enter' && openLogEdit(log)}
            >
              <td class="col-project" on:click|stopPropagation={() => openProjectEdit(log)}>
                <span class="project-title">{getProjectPath(log, $projects)}</span>
                {#if projBadges.length > 0}
                  <div class="badge-row">
                    {#each projBadges as b}
                      <Badge label={b.label} catType={b.catType} selected={true} clickable={false} size="sm" />
                    {/each}
                  </div>
                {/if}
              </td>
              <td class="col-title">
                <span class="log-title">{log.title}</span>
                {#if typeMeta}
                  <span class="log-type">{typeMeta}</span>
                {/if}
                {#if logBadges.length > 0}
                  <div class="badge-row">
                    {#each logBadges as b}
                      <Badge label={b.label} catType={b.catType} selected={true} clickable={false} size="sm" />
                    {/each}
                  </div>
                {/if}
              </td>
              <td class="col-deadline">
                {#if log.due_date}
                  <span class="deadline-pill" style="background:{dl}; color:{dlText}">{log.due_date}</span>
                {:else}
                  <span class="no-date">—</span>
                {/if}
              </td>
              <td class="col-desc">
                {#if log.description}
                  <div class="log-desc" on:click={handleLinkClick}>{@html log.description}</div>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table></div>
    {/if}
  </div>
  </div>

  {#if showLogEditor}
    <LogEditor
      log={editorLog}
      {logTypes} {cat1Vals} {cat2Vals} {cat3Vals} {cat4Vals}
      cat1Label={$settings.category1_label} cat2Label={$settings.category2_label}
      cat3Label={$settings.category3_label} cat4Label={$settings.category4_label}
      allProjects={nonTemplateProjects}
      on:close={() => { showLogEditor = false; editorLog = null; }}
    />
  {/if}

  {#if showProjectEditor}
    <ProjectEditor
      project={editorProject}
      allProjects={nonTemplateProjects}
      {cat1Vals} {cat2Vals} {cat3Vals} {cat4Vals}
      cat1Label={$settings.category1_label} cat2Label={$settings.category2_label}
      cat3Label={$settings.category3_label} cat4Label={$settings.category4_label}
      on:close={() => { showProjectEditor = false; editorProject = null; }}
    />
  {/if}
</div>

<style>
  .page {
    display: flex; flex-direction: column;
    height: 100%; overflow: hidden;
    background: var(--bg); color: var(--text);
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
  .sidebar-cat { flex: 1; overflow-y: auto; padding: var(--sp-sidebar-pad, 16px); min-height: 0; }
  .sidebar-separator { height: 1px; background: var(--border); flex-shrink: 0; margin: 0 12px; }

  /* Filter controls — same look as the homepage menu bar */
  .toggle-wrap {
    display: flex; align-items: center; gap: 7px; cursor: pointer; user-select: none;
  }
  .toggle-label { font-size: 13px; font-weight: 600; color: var(--text-muted); }
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

  .project-lookup { position: relative; display: flex; align-items: center; }
  .project-lookup-input {
    background: var(--surface-2); color: var(--text);
    border: 1px solid var(--border); border-radius: 8px;
    padding: 6px 28px 6px 10px; font-size: 13px; font-family: inherit;
    outline: none; width: 180px;
    transition: border-color 0.15s;
  }
  .project-lookup.active .project-lookup-input,
  .project-lookup-input:focus { border-color: var(--accent); }
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

  .table-wrap {
    grid-column: 2; grid-row: 1;
    overflow-y: auto;
    padding: 16px 20px;
    min-height: 0;
  }

  .table-outer {
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden;
  }

  .log-table {
    width: 100%; border-collapse: collapse; table-layout: fixed;
  }

  .log-table thead th {
    padding: 5px 8px;
    text-align: left;
    font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em;
    color: var(--text-muted);
    background: var(--surface-3);
    border-bottom: 1px solid var(--border);
  }
  .log-table thead th + th { border-left: 1px solid var(--border); }

  .log-row { transition: background 0.1s; cursor: pointer; }
  .log-row:hover td { background: var(--surface-2); }
  .log-row.log-closed { opacity: 0.55; }

  .log-row td {
    padding: 6px 8px;
    vertical-align: top;
    border-bottom: 1px solid var(--border);
    border-right: 1px solid var(--border);
    font-size: 13px;
  }
  .log-row td:last-child { border-right: none; }
  .log-row:last-child td { border-bottom: none; }

  .col-project { width: 20%; }
  .col-title { width: 25%; }
  .col-deadline { width: 110px; }
  .col-desc { }

  .project-title { display: block; font-size: 12px; color: var(--text-muted); }
  .log-title { display: block; font-weight: 600; color: var(--text); }
  .log-type { display: block; font-size: 11px; color: var(--text-muted); margin-top: 2px; }

  .badge-row {
    display: flex; flex-wrap: wrap; gap: 3px; margin-top: 4px;
  }
  .deadline-pill {
    display: inline-block;
    font-size: 11px; font-weight: 600;
    border-radius: 6px; padding: 2px 8px; white-space: nowrap;
  }

  .no-date { color: var(--text-muted); }

  .log-desc {
    font-size: 12px; color: var(--text-muted); line-height: 1.5;
  }
  .log-desc :global(*) { margin: 0; }
  .log-desc :global(ul), .log-desc :global(ol) { padding-left: 16px; }

  .empty { font-size: 13px; color: var(--text-muted); padding: 20px 0; }
</style>
