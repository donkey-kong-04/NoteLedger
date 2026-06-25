<script lang="ts">
  import { onMount } from 'svelte';
  import { logs, projects, picklists, loadAll } from '$lib/store';
  import { deadlineColor, contrastText, handleLinkClick } from '$lib/types';
  import type { Log, Project, PicklistValue } from '$lib/types';

  onMount(() => loadAll());

  function getProject(log: Log): Project | undefined {
    return $projects.find(p => p.id === log.project_id);
  }

  function getProjectTitle(log: Log): string {
    const p = getProject(log);
    if (!p) return '—';
    // Build full path for nested projects
    const parts: string[] = [p.title];
    let cur = p;
    while (cur.parent_id) {
      const parent = $projects.find(x => x.id === cur.parent_id);
      if (!parent) break;
      parts.unshift(parent.title);
      cur = parent;
    }
    return parts.join(' › ');
  }

  $: openLogs = $logs.filter(l => !l.is_closed);

  $: sortedLogs = [...openLogs].sort((a, b) => {
    if (!a.due_date && !b.due_date) return 0;
    if (!a.due_date) return 1;
    if (!b.due_date) return -1;
    return b.due_date.localeCompare(a.due_date);
  });
</script>

<div class="page">
  <header class="page-header">
    <a href="/" class="back-btn">← Back</a>
    <h1>Deadlines</h1>
    <span class="count">{sortedLogs.length} open log{sortedLogs.length !== 1 ? 's' : ''}</span>
  </header>

  <div class="table-wrap">
    {#if sortedLogs.length === 0}
      <p class="empty">No open logs.</p>
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
            {@const dl = log.due_date ? deadlineColor(log.due_date) : null}
            {@const dlText = dl ? contrastText(dl) : '#fff'}
            <tr class="log-row">
              <td class="col-project">
                <span class="project-title">{getProjectTitle(log)}</span>
              </td>
              <td class="col-title">
                <span class="log-title">{log.title}</span>
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
                  <div class="log-desc" on:click|stopPropagation={handleLinkClick}>{@html log.description}</div>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table></div>
    {/if}
  </div>
</div>

<style>
  .page {
    display: flex; flex-direction: column;
    height: 100vh; overflow: hidden;
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

  .back-btn {
    font-size: 13px; font-weight: 600;
    color: var(--accent); text-decoration: none;
    padding: 4px 10px; border-radius: 6px;
    border: 1px solid var(--accent);
    transition: background 0.15s;
  }
  .back-btn:hover { background: rgba(99,102,241,0.1); }

  h1 { margin: 0; font-size: 18px; font-weight: 700; }

  .count { font-size: 12px; color: var(--text-muted); }

  .table-wrap {
    flex: 1; overflow-y: auto;
    padding: 16px 20px;
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

  .log-row { transition: background 0.1s; }
  .log-row:hover td { background: var(--surface-2); }

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

  .project-title { font-size: 12px; color: var(--text-muted); }
  .log-title { font-weight: 600; color: var(--text); }

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
