<script lang="ts">
  import Badge from './Badge.svelte';
  import { createEventDispatcher } from 'svelte';
  import { deadlineColor, contrastText, handleLinkClick, openSinceLabel } from '../types';
  import type { Log, Project, PicklistValue, ProjectLink } from '../types';
  import { openLink } from '../types';
  import { sanitizeHtml } from '../sanitize';

  export let project: Project;
  export let subProjects: Project[];
  export let allLogs: Log[];
  export let allLogsTotal: Log[] = [];
  export let allProjects: Project[];
  export let visibleProjectIds: Set<number> = new Set();
  export let ancestorOnlyProjectIds: Set<number> = new Set();
  export let showClosed: boolean = false;
  export let depth: number = 0;
  export let collapseSignal: number = 0;
  export let collapseAll: boolean = false;
  export let allLinks: ProjectLink[] = [];
  // Templates page: shows a Clone button on top-level template cards.
  export let showCloneButton: boolean = false;
  export let logTypes: PicklistValue[];
  export let cat1Vals: PicklistValue[];
  export let cat2Vals: PicklistValue[];
  export let cat3Vals: PicklistValue[];
  export let cat4Vals: PicklistValue[];

  const dispatch = createEventDispatcher();
  // Sub-project cards unmount while their parent is folded; start from the
  // global fold state so they come back folded after a "Fold all".
  let collapsed = collapseAll;
  let showTypePicker = false;
  let activeTab: 'logs' | 'links' = 'logs';

  // Apply a global fold/unfold only when the signal changes — leaves the local
  // chevron toggle untouched in between.
  let lastCollapseSignal = collapseSignal;
  $: if (collapseSignal !== lastCollapseSignal) {
    lastCollapseSignal = collapseSignal;
    collapsed = collapseAll;
  }

  function pickType(typeId: number) {
    showTypePicker = false;
    dispatch('newLogInProject', { project, typeId });
  }

  function handlePickerKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') showTypePicker = false;
  }

  $: ownLogs = allLogs
    .filter(l => Number(l.project_id) === Number(project.id))
    .sort((a, b) => {
      // open with date ASC, then open without date, then closed with date ASC
      if (a.is_closed !== b.is_closed) return a.is_closed ? 1 : -1;
      if (!a.due_date && !b.due_date) return 0;
      if (!a.due_date) return 1;
      if (!b.due_date) return -1;
      return a.due_date.localeCompare(b.due_date);
    });

  $: ownLogsTotal = allLogsTotal.filter(l => Number(l.project_id) === Number(project.id));
  $: openCount = ownLogsTotal.filter(l => !l.is_closed).length;
  $: countLabel = `${openCount} / ${ownLogsTotal.length}`;

  $: visibleSubProjects = subProjects.filter(s => visibleProjectIds.has(s.id));

  $: projectCatBadges = [
    ...(project.category1_ids ?? []).map(id => ({ val: cat1Vals.find(v => v.id === id), catType: 'category_1' })),
    ...(project.category2_ids ?? []).map(id => ({ val: cat2Vals.find(v => v.id === id), catType: 'category_2' })),
    ...(project.category3_ids ?? []).map(id => ({ val: cat3Vals.find(v => v.id === id), catType: 'category_3' })),
    ...(project.category4_ids ?? []).map(id => ({ val: cat4Vals.find(v => v.id === id), catType: 'category_4' })),
  ].filter(x => x.val) as { val: PicklistValue; catType: string }[];

  function getLogType(log: Log) {
    return logTypes.find(v => v.id === log.type_id);
  }

  function getCatBadges(log: Log) {
    return [
      ...log.category1_ids.map(id => ({ val: cat1Vals.find(v => v.id === id), type: 'category_1' })),
      ...log.category2_ids.map(id => ({ val: cat2Vals.find(v => v.id === id), type: 'category_2' })),
      ...log.category3_ids.map(id => ({ val: cat3Vals.find(v => v.id === id), type: 'category_3' })),
      ...log.category4_ids.map(id => ({ val: cat4Vals.find(v => v.id === id), type: 'category_4' })),
    ].filter(x => x.val) as { val: PicklistValue; type: string }[];
  }

  $: ownLinks = allLinks.filter(l => l.project_id === project.id);
</script>

<!-- The node wraps the card and its sub-tree so connector lines can be drawn
     between a parent and each of its children. -->
<div class="node">
<div class="project-block" class:closed={project.is_closed}>
  <div class="project-header" style="padding-left: 10px">
    <button class="chevron" class:collapsed on:click={() => collapsed = !collapsed} aria-label="Toggle">
      <svg width="10" height="10" viewBox="0 0 10 10" fill="none">
        <path d={collapsed ? 'M3 2.5l4 2.5-4 2.5' : 'M2.5 3l2.5 4 2.5-4'}
          stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </button>
    <div class="project-title-group">
      <span class="project-title"
        role="button" tabindex="0"
        on:click={() => dispatch('editProject', project)}
        on:keydown={e => e.key === 'Enter' && dispatch('editProject', project)}
      >{project.title}</span>
      {#each projectCatBadges as badge}
        <Badge label={badge.val.label} catType={badge.catType} selected={true} clickable={false} size="sm" />
      {/each}
    </div>
    {#if project.is_closed}
      <span class="closed-pill">Closed</span>
    {/if}
    <span class="count-badge">{countLabel}</span>
    {#if showCloneButton}
      <button class="clone-btn" on:click={() => dispatch('cloneProject', project)} title="Clone this template into a new project">⧉ Clone</button>
    {/if}
    <button class="sub-btn" on:click={() => dispatch('newSubProject', project)} title="New sub-project">＋ Sub-project</button>
    <button class="sub-btn" on:click={() => dispatch('newLink', project)} title="Add link">＋ Link</button>
    <div class="add-wrap">
      <button class="add-btn" on:click={() => showTypePicker = !showTypePicker} title="Add log">+</button>
      {#if showTypePicker}
        <div class="type-picker" on:keydown={handlePickerKeydown}>
          {#each logTypes as lt}
            <button class="type-option" on:click={() => pickType(lt.id)}>{lt.label}</button>
          {/each}
        </div>
        <div class="type-picker-backdrop" on:click={() => showTypePicker = false} role="presentation"></div>
      {/if}
    </div>
  </div>

  {#if !collapsed}
    <!-- Body: Logs / Links tabs (sub-projects are outside, always visible) -->
    {#if (ownLogs.length > 0 || ownLinks.length > 0) && !ancestorOnlyProjectIds.has(project.id)}
      <div class="card-body">
        <div class="body-tabs">
          <button class="body-tab" class:active={activeTab === 'logs'} on:click={() => activeTab = 'logs'}>
            Logs {#if ownLogs.length > 0}<span class="tab-count">{ownLogs.length}</span>{/if}
          </button>
          <button class="body-tab" class:active={activeTab === 'links'} on:click={() => activeTab = 'links'}>
            Links {#if ownLinks.length > 0}<span class="tab-count">{ownLinks.length}</span>{/if}
          </button>
        </div>

        {#if activeTab === 'logs'}
        {#if ownLogs.length > 0}
          <div class="log-table-wrap">
            <table class="log-table">
              <thead>
                <tr>
                  <th class="col-title">Title</th>
                  <th class="col-deadline">Deadline</th>
                  <th class="col-desc">Description</th>
                </tr>
              </thead>
              <tbody>
                {#each ownLogs as log (log.id)}
                  {@const dl = log.due_date && !log.is_closed ? deadlineColor(log.due_date) : null}
                  {@const dlText = dl ? contrastText(dl) : '#fff'}
                  {@const badges = getCatBadges(log)}
                  {@const type = getLogType(log)}
                  {@const typeMeta = [type?.label, openSinceLabel(log)].filter(Boolean).join(' · ')}
                  <tr
                    class="log-row"
                    class:log-closed={log.is_closed}
                    on:click={() => dispatch('edit', log)}
                    role="button" tabindex="0"
                    on:keydown={e => e.key === 'Enter' && dispatch('edit', log)}
                  >
                    <td class="col-title">
                      <span class="log-title">{log.title}</span>
                      {#if typeMeta}
                        <span class="log-type">{typeMeta}</span>
                      {/if}
                      {#if badges.length > 0}
                        <div class="log-badges">
                          {#each badges as badge}
                            <Badge label={badge.val.label} catType={badge.type} selected={true} clickable={false} size="sm" />
                          {/each}
                        </div>
                      {/if}
                    </td>
                    <td class="col-deadline">
                      {#if log.due_date}
                        <span class="deadline-pill" style="background:{dl}; color:{dlText}">{log.due_date}</span>
                      {/if}
                    </td>
                    <td class="col-desc">
                      {#if log.description}
                        <div class="log-desc" on:click={handleLinkClick}>{@html sanitizeHtml(log.description)}</div>
                      {/if}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        {:else}
          <p class="tab-empty">No logs.</p>
        {/if}
        {:else}
        {#if ownLinks.length > 0}
          <div class="links-panel">
            {#each ownLinks as link (link.id)}
              <div class="link-row">
                <button class="link-card" on:click|stopPropagation={() => openLink(link.url).catch(e => console.error('openLink failed:', e, link.url))}>
                  <svg class="link-icon" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <path d="M6.5 9.5a3.5 3.5 0 0 0 5 0l2-2a3.536 3.536 0 0 0-5-5L7.5 3.5" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
                    <path d="M9.5 6.5a3.5 3.5 0 0 0-5 0l-2 2a3.536 3.536 0 0 0 5 5l1-1" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"/>
                  </svg>
                  <span class="link-card-label">{link.label}</span>
                </button>
                <button class="link-action" on:click|stopPropagation={() => dispatch('editLink', link)} title="Edit">✎</button>
              </div>
            {/each}
          </div>
        {:else}
          <p class="tab-empty">No links — use ＋ Link to add one.</p>
        {/if}
        {/if}
      </div>
    {/if}

    {#if ownLogs.length === 0 && ownLinks.length === 0 && visibleSubProjects.length === 0 && !ancestorOnlyProjectIds.has(project.id)}
      {#if ownLogsTotal.length > 0}
        <p class="empty">No logs matching the filters.</p>
      {:else}
        <p class="empty">No logs yet — click <strong>+</strong> to add one.</p>
      {/if}
    {/if}
  {/if}
</div>

{#if !collapsed && visibleSubProjects.length > 0}
  <div class="sub-tree">
    {#each visibleSubProjects as sub (sub.id)}
      <svelte:self
        project={sub}
        subProjects={allProjects.filter(p => p.parent_id != null && Number(p.parent_id) === Number(sub.id))}
        {allLogs} {allLogsTotal} {allProjects} {allLinks}
        {visibleProjectIds} {ancestorOnlyProjectIds} {showClosed}
        {collapseSignal} {collapseAll}
        depth={depth + 1}
        {logTypes} {cat1Vals} {cat2Vals} {cat3Vals} {cat4Vals}
        on:edit
        on:editProject
        on:newLogInProject
        on:newSubProject
        on:newLink
        on:editLink
      />
    {/each}
  </div>
{/if}
</div>

<style>
  .node { position: relative; }

  /* Sub-projects: indented under the parent, with tree connector lines
     (vertical trunk + an elbow into each child's header). */
  .sub-tree {
    position: relative;
    margin-left: 14px;
    padding-left: 18px;
    display: flex;
    flex-direction: column;
  }
  .sub-tree > :global(.node) { padding-top: var(--sp-card-gap, 12px); }
  /* Elbow into the child's header */
  .sub-tree > :global(.node)::before {
    content: '';
    position: absolute;
    left: -18px;
    top: calc(var(--sp-card-gap, 12px) + 18px);
    width: 16px;
    border-top: 2px solid var(--project-header, var(--surface-3));
  }
  /* Vertical trunk — runs the full height so it reaches the next sibling… */
  .sub-tree > :global(.node)::after {
    content: '';
    position: absolute;
    left: -18px;
    top: 0;
    bottom: 0;
    border-left: 2px solid var(--project-header, var(--surface-3));
  }
  /* …but stops at the elbow on the last child. */
  .sub-tree > :global(.node:last-child)::after {
    bottom: auto;
    height: calc(var(--sp-card-gap, 12px) + 19px);
  }

  .project-block {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
  }
  .project-block.closed { opacity: 0.55; }

  .project-header {
    display: flex; align-items: center; gap: 8px;
    padding-top: 7px; padding-bottom: 7px; padding-right: 10px;
    background: var(--project-header, var(--surface-3));
    border-radius: 7px 7px 0 0;
    min-height: 36px;
  }

  .chevron {
    flex-shrink: 0;
    width: 18px; height: 18px;
    background: none; border: none; cursor: pointer;
    color: var(--text-muted); border-radius: 4px;
    display: flex; align-items: center; justify-content: center;
    padding: 0; transition: background 0.1s, color 0.1s;
  }
  .chevron:hover { background: var(--surface-3); color: var(--text); }

  .project-title-group {
    flex: 1; min-width: 0;
    display: flex; align-items: center; gap: 6px; flex-wrap: wrap;
  }
  .project-title {
    font-size: 13px; font-weight: 700;
    color: var(--text); cursor: pointer;
    white-space: nowrap;
  }
  .project-title:hover { color: var(--accent); }

  .closed-pill {
    font-size: 10px; font-weight: 600; text-transform: uppercase;
    background: var(--border); color: var(--text-muted);
    border-radius: 999px; padding: 1px 7px; flex-shrink: 0;
  }

  .count-badge {
    font-size: 11px; color: var(--text-muted);
    background: var(--surface-3); border-radius: 999px;
    padding: 1px 7px; font-weight: 600; flex-shrink: 0;
  }

  .sub-btn {
    font-size: 11px; color: var(--text-muted);
    background: none; border: 1.5px dashed var(--border);
    border-radius: 999px; padding: 2px 8px;
    cursor: pointer; font-family: inherit; flex-shrink: 0;
    transition: color 0.15s, border-color 0.15s;
  }
  .sub-btn:hover { color: var(--text); border-color: var(--text-muted); }

  .clone-btn {
    font-size: 11px; font-weight: 600; color: #fff;
    background: var(--accent); border: 1.5px solid var(--accent);
    border-radius: 999px; padding: 2px 10px;
    cursor: pointer; font-family: inherit; flex-shrink: 0;
    transition: opacity 0.15s;
  }
  .clone-btn:hover { opacity: 0.85; }

  .add-wrap { position: relative; flex-shrink: 0; }

  .add-btn {
    width: 22px; height: 22px;
    background: var(--accent); color: #fff;
    border: none; border-radius: 5px;
    cursor: pointer; font-size: 16px; line-height: 1;
    display: flex; align-items: center; justify-content: center;
    transition: opacity 0.1s; padding: 0;
  }
  .add-btn:hover { opacity: 0.85; }

  .type-picker-backdrop {
    position: fixed; inset: 0; z-index: 199;
  }
  .type-picker {
    position: absolute; top: calc(100% + 4px); right: 0;
    background: var(--accent);
    border: none;
    border-radius: 8px;
    box-shadow: 0 4px 16px rgba(0,0,0,0.25);
    z-index: 200;
    min-width: 120px;
    overflow: hidden;
  }
  .type-option {
    display: block; width: 100%;
    padding: 8px 14px; text-align: left;
    background: none; border: none; cursor: pointer;
    font-size: 13px; color: #fff; font-family: inherit;
    transition: background 0.1s;
  }
  .type-option:hover { background: rgba(255,255,255,0.15); }

  /* Log table */
  .log-table-wrap {
    padding: 8px 10px;
    box-sizing: border-box;
  }

  .log-table {
    width: 100%; border-collapse: collapse; table-layout: fixed;
    border: 1px solid var(--border);
    border-radius: 6px;
    overflow: hidden; /* works because log-table-wrap has no overflow:hidden */
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

  .log-row {
    cursor: pointer;
    transition: background 0.1s;
  }
  .log-row:hover td { background: var(--surface-2); }
  .log-row.log-closed td { background: var(--surface-3); opacity: 0.6; }

  .log-row td {
    padding: 6px 8px;
    vertical-align: top;
    border-bottom: 1px solid var(--border);
    border-right: 1px solid var(--border);
    font-size: 13px;
  }
  .log-row td:last-child { border-right: none; }
  .log-row:last-child td { border-bottom: none; }

  .col-title { width: 30%; }
  .col-deadline { width: 110px; }
  .col-desc { }

  .log-title { display: block; font-weight: 600; color: var(--text); font-size: 13px; line-height: 1.3; }
  .log-type { display: block; font-size: 11px; color: var(--text-muted); margin-top: 2px; }
  .log-badges { display: flex; flex-wrap: wrap; gap: 3px; margin-top: 4px; }

  .deadline-pill {
    display: inline-block;
    font-size: 11px; font-weight: 600;
    border-radius: 6px; padding: 2px 8px; white-space: nowrap;
  }

  .log-desc {
    font-size: 12px; color: var(--text-muted); line-height: 1.5;
  }
  .log-desc :global(*) { margin: 0; }
  .log-desc :global(ul), .log-desc :global(ol) { padding-left: 16px; }

  .empty {
    font-size: 12px; color: var(--text-muted);
    margin: 0; padding: 8px 10px;
    box-sizing: border-box;
  }

  .card-body {
    display: flex;
    flex-direction: column;
  }
  .card-body .log-table-wrap { min-width: 0; }

  /* Logs / Links tabs — folder-tab style: the active tab shares the card
     body's background and merges with the panel below (its bottom border goes
     transparent), like a physical folder tab; inactive tabs stay recessed. */
  .body-tabs {
    display: flex;
    /* Same color as the project header bar so the card top reads as one piece
       (in dark mode --project-header and --surface-3 are already identical). */
    background: var(--project-header, var(--surface-3));
    padding: 0;
    gap: 0;
  }
  /* No separator borders: tabs are distinguished by background alone (active
     = card surface, inactive = header-colored strip), same as dark mode where
     the old borders were invisible anyway (--border == --project-header). */
  .body-tab {
    display: inline-flex; align-items: center; justify-content: center; gap: 6px;
    background: none; cursor: pointer; font-family: inherit;
    font-size: 12px; font-weight: 600;
    color: var(--text-muted);
    padding: 7px 16px;
    border: none;
    border-radius: 7px 7px 0 0;
    transition: background 0.15s, color 0.15s;
  }
  .body-tab:hover { color: var(--text); background: var(--surface-hover); }
  .body-tab.active {
    background: var(--surface);
    color: var(--text);
    box-shadow: inset 0 2px 0 var(--accent);
  }
  .body-tab.active:hover { color: var(--text); background: var(--surface); }
  .tab-count {
    font-size: 10px; font-weight: 700;
    background: var(--surface); color: var(--text-muted);
    border-radius: 999px; padding: 0 6px; line-height: 14px;
  }
  .body-tab.active .tab-count { background: var(--surface-3); color: var(--text); }

  .tab-empty {
    margin: 0; padding: 10px 14px;
    font-size: 12px; color: var(--text-muted);
  }

  /* Links panel — full-width row of link cards, wrapping */
  .links-panel {
    padding: 6px 8px;
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    min-width: 0;
  }

  .link-row {
    display: flex; align-items: center; gap: 4px;
    min-width: 0;
  }

  .link-card {
    flex: 1; min-width: 0;
    display: inline-flex; align-items: center; gap: 5px;
    padding: 3px 7px 3px 5px;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 4px;
    cursor: pointer;
    font-family: inherit;
    transition: background 0.12s, border-color 0.12s;
    max-width: 100%;
  }
  .link-card:hover {
    background: var(--surface-2);
    border-color: var(--accent);
  }

  .link-icon {
    width: 13px; height: 13px;
    flex-shrink: 0;
    color: var(--accent);
  }

  .link-card-label {
    font-size: 12px; font-weight: 500;
    color: var(--accent);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
    min-width: 0;
  }

  .link-action {
    flex-shrink: 0;
    background: none; border: none; cursor: pointer;
    color: var(--text-muted); font-size: 13px; padding: 0 2px;
  }
  .link-action:hover { color: var(--text); }
</style>
