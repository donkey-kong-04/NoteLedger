<script lang="ts">
  import Badge from './Badge.svelte';
  import { createEventDispatcher } from 'svelte';
  import { deadlineColor, contrastText, handleLinkClick } from '../types';
  import type { Log, Project, PicklistValue } from '../types';

  export let project: Project;
  export let subProjects: Project[];
  export let allLogs: Log[];
  export let allLogsTotal: Log[] = [];
  export let allProjects: Project[];
  export let visibleProjectIds: Set<number> = new Set();
  export let ancestorOnlyProjectIds: Set<number> = new Set();
  export let showClosed: boolean = false;
  export let depth: number = 0;
  export let logTypes: PicklistValue[];
  export let cat1Vals: PicklistValue[];
  export let cat2Vals: PicklistValue[];
  export let cat3Vals: PicklistValue[];
  export let cat4Vals: PicklistValue[];

  const dispatch = createEventDispatcher();
  let collapsed = false;
  let hoveredLogId: number | null = null;
  let showTypePicker = false;

  function pickType(typeId: number) {
    showTypePicker = false;
    dispatch('newLogInProject', { project, typeId });
  }

  function handlePickerKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') showTypePicker = false;
  }

  const INDENT = 24; // px per depth level

  $: ownLogs = allLogs
    .filter(l => Number(l.project_id) === Number(project.id))
    .sort((a, b) => {
      if (!a.due_date && !b.due_date) return 0;
      if (!a.due_date) return 1;
      if (!b.due_date) return -1;
      return b.due_date.localeCompare(a.due_date);
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
</script>

<!-- Project header row -->
<div class="project-block" class:closed={project.is_closed} style="margin-left: {depth * INDENT}px">
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
    <!-- Log table — hidden for ancestor-context projects -->
    {#if ownLogs.length > 0 && !ancestorOnlyProjectIds.has(project.id)}
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
              {@const dl = log.due_date ? deadlineColor(log.due_date) : null}
              {@const dlText = dl ? contrastText(dl) : '#fff'}
              {@const badges = getCatBadges(log)}
              {@const type = getLogType(log)}
              <tr
                class="log-row"
                class:log-closed={log.is_closed}
                on:click={() => dispatch('edit', log)}
                on:mouseenter={() => hoveredLogId = log.id}
                on:mouseleave={() => hoveredLogId = null}
                role="button" tabindex="0"
                on:keydown={e => e.key === 'Enter' && dispatch('edit', log)}
              >
                <td class="col-title">
                  <span class="log-title">{log.title}</span>
                  {#if type}
                    <span class="log-type">{type.label}</span>
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
                    <div class="log-desc" class:expanded={hoveredLogId === log.id} on:click|stopPropagation={handleLinkClick}>{@html log.description}</div>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}

    {#if ownLogs.length === 0 && visibleSubProjects.length === 0 && !ancestorOnlyProjectIds.has(project.id)}
      <p class="empty">No logs yet — click <strong>+</strong> to add one.</p>
    {/if}
  {/if}
</div>

<!-- Sub-projects rendered at same DOM level, just deeper indent -->
{#if !collapsed}
  {#each visibleSubProjects as sub (sub.id)}
    <svelte:self
      project={sub}
      subProjects={allProjects.filter(p => p.parent_id != null && Number(p.parent_id) === Number(sub.id))}
      {allLogs} {allLogsTotal} {allProjects}
      {visibleProjectIds} {ancestorOnlyProjectIds} {showClosed}
      depth={depth + 1}
      {logTypes} {cat1Vals} {cat2Vals} {cat3Vals} {cat4Vals}
      on:edit
      on:editProject
      on:newLogInProject
    />
  {/each}
{/if}

<style>
  .project-block {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
  }
  .project-block.closed { opacity: 0.55; }

  .project-header {
    display: flex; align-items: center; gap: 8px;
    padding-top: 7px; padding-bottom: 7px; padding-right: 10px;
    background: var(--surface-2);
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
    max-height: 3.6em; overflow: hidden;
    display: -webkit-box; -webkit-box-orient: vertical; -webkit-line-clamp: 3;
    transition: max-height 0.2s ease;
  }
  .log-desc.expanded {
    max-height: 2000px;
    -webkit-line-clamp: unset;
    display: block;
  }
  .log-desc :global(*) { margin: 0; }
  .log-desc :global(ul), .log-desc :global(ol) { padding-left: 16px; }

  .empty {
    font-size: 12px; color: var(--text-muted);
    margin: 0; padding-top: 8px; padding-bottom: 8px; padding-right: 10px;
    box-sizing: border-box;
  }
</style>
