<script lang="ts">
  import LogCard from './LogCard.svelte';
  import Badge from './Badge.svelte';
  import { createEventDispatcher } from 'svelte';
  import type { Log, Project, PicklistValue } from '../types';

  export let project: Project;
  export let subProjects: Project[];
  export let allLogs: Log[];
  export let allLogsTotal: Log[] = [];
  export let allProjects: Project[];
  export let filtersActive: boolean = false;
  export let showAllSubProjects: boolean = false;
  export let logTypes: PicklistValue[];
  export let cat1Vals: PicklistValue[];
  export let cat2Vals: PicklistValue[];
  export let cat3Vals: PicklistValue[];
  export let cat4Vals: PicklistValue[];

  $: catBadges = [
    ...(project.category1_ids ?? []).map(id => ({ val: cat1Vals.find(v => v.id === id), type: 'category_1' })),
    ...(project.category2_ids ?? []).map(id => ({ val: cat2Vals.find(v => v.id === id), type: 'category_2' })),
    ...(project.category3_ids ?? []).map(id => ({ val: cat3Vals.find(v => v.id === id), type: 'category_3' })),
    ...(project.category4_ids ?? []).map(id => ({ val: cat4Vals.find(v => v.id === id), type: 'category_4' })),
  ].filter(x => x.val) as { val: PicklistValue; type: string }[];

  const dispatch = createEventDispatcher();
  let collapsed = false;

  $: ownLogs = allLogs.filter(l => Number(l.project_id) === Number(project.id));
  $: ownLogsTotal = allLogsTotal.filter(l => Number(l.project_id) === Number(project.id));
  $: countLabel = ownLogsTotal.length === 0 ? '0' :
    ownLogs.length === ownLogsTotal.length
      ? `${ownLogs.length}`
      : `${ownLogs.filter(l => !l.is_closed).length} / ${ownLogsTotal.length}`;

  function subHasAnyLogs(projectId: number): boolean {
    if (allLogs.some(l => Number(l.project_id) === Number(projectId))) return true;
    return allProjects
      .filter(p => p.parent_id != null && Number(p.parent_id) === Number(projectId))
      .some(sub => subHasAnyLogs(sub.id));
  }
</script>

<div class="project-card">
  <div class="project-header">
    <button class="collapse-btn" on:click={() => collapsed = !collapsed} aria-label="Toggle">
      <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
        <path d={collapsed ? 'M3 4.5L6 7.5L9 4.5' : 'M4.5 9L7.5 6L4.5 3'}
          stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
      </svg>
    </button>
    <div class="project-title-group">
      <span class="project-title"
        role="button" tabindex="0"
        on:click={() => dispatch('editProject', project)}
        on:keydown={e => e.key === 'Enter' && dispatch('editProject', project)}
      >{project.title}</span>
      {#if catBadges.length > 0}
        <div class="project-cat-badges">
          {#each catBadges as { val, type }}
            <Badge label={val.label} catType={type} selected={true} clickable={false} size="sm" />
          {/each}
        </div>
      {/if}
    </div>
    <span class="log-count">{countLabel}</span>
    <button class="add-log-btn" on:click={() => dispatch('newLogInProject', project)} title="Add log to project">+</button>
  </div>

  {#if !collapsed}
    <div class="project-body">
      {#each ownLogs as log (log.id)}
        <LogCard {log} {logTypes} {cat1Vals} {cat2Vals} {cat3Vals} {cat4Vals} on:edit />
      {/each}

      {#each subProjects as sub (sub.id)}
        {#if showAllSubProjects || subHasAnyLogs(sub.id)}
          <svelte:self
            project={sub}
            subProjects={allProjects.filter(p => p.parent_id != null && Number(p.parent_id) === Number(sub.id))}
            {allLogs}
            {allLogsTotal}
            {allProjects} {filtersActive} {showAllSubProjects} {logTypes} {cat1Vals} {cat2Vals} {cat3Vals} {cat4Vals}
            on:edit
            on:editProject
            on:newLogInProject
          />
        {/if}
      {/each}

      {#if subProjects.length === 0 && ownLogs.length === 0}
        <p class="empty-project">No logs yet — click <strong>+</strong> to add one.</p>
      {/if}
    </div>
  {/if}
</div>

<style>
  .project-card {
    border: 1.5px solid var(--border);
    border-radius: 12px;
    overflow: hidden;
    background: var(--surface);
    grid-column: 1 / -1;
  }

  .project-header {
    display: flex; align-items: center; gap: 8px;
    padding: 10px 14px;
    background: var(--surface-2);
    border-bottom: 1px solid var(--border);
  }

  .collapse-btn {
    width: 22px; height: 22px; background: none; border: none;
    color: var(--text-muted); cursor: pointer; border-radius: 4px;
    display: flex; align-items: center; justify-content: center;
    flex-shrink: 0; transition: background 0.1s; padding: 0;
  }
  .collapse-btn:hover { background: var(--surface-3); color: var(--text); }

  .project-title-group {
    flex: 1; display: flex; flex-wrap: wrap; align-items: center; gap: 4px; min-width: 0;
  }

  .project-title {
    font-size: 13px; font-weight: 700;
    color: var(--text); cursor: pointer; letter-spacing: 0.01em;
  }
  .project-title:hover { color: var(--accent); }

  .project-cat-badges { display: flex; flex-wrap: wrap; gap: 3px; }

  .log-count {
    font-size: 11px; color: var(--text-muted);
    background: var(--surface-3); border-radius: 999px;
    padding: 1px 7px; font-weight: 600;
  }

  .add-log-btn {
    width: 22px; height: 22px; background: none;
    border: 1px solid var(--border); border-radius: 5px; cursor: pointer;
    color: var(--text-muted); font-size: 16px; line-height: 1;
    display: flex; align-items: center; justify-content: center;
    transition: background 0.1s, color 0.1s; flex-shrink: 0; padding: 0;
  }
  .add-log-btn:hover { background: var(--accent); color: #fff; border-color: var(--accent); }

  .project-body {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    grid-auto-rows: min-content;
    align-content: start;
    gap: 8px; padding: 12px;
  }

  .empty-project {
    font-size: 12px; color: var(--text-muted);
    margin: 0; text-align: center; padding: 8px 0;
    grid-column: 1 / -1;
  }
</style>
