<script lang="ts">
  import Badge from './Badge.svelte';
  import { deadlineColor, contrastText, CAT_COLORS, handleLinkClick } from '../types';
  import { sanitizeHtml } from '../sanitize';
  import type { Log, PicklistValue } from '../types';
  import { createEventDispatcher } from 'svelte';

  export let log: Log;
  export let logTypes: PicklistValue[];
  export let cat1Vals: PicklistValue[];
  export let cat2Vals: PicklistValue[];
  export let cat3Vals: PicklistValue[];
  export let cat4Vals: PicklistValue[];

  const dispatch = createEventDispatcher();

  $: logType = logTypes.find(v => v.id === log.type_id);

  $: catBadges = [
    ...log.category1_ids.map(id => ({ val: cat1Vals.find(v => v.id === id), type: 'category_1' })),
    ...log.category2_ids.map(id => ({ val: cat2Vals.find(v => v.id === id), type: 'category_2' })),
    ...log.category3_ids.map(id => ({ val: cat3Vals.find(v => v.id === id), type: 'category_3' })),
    ...log.category4_ids.map(id => ({ val: cat4Vals.find(v => v.id === id), type: 'category_4' })),
  ].filter(x => x.val) as { val: PicklistValue; type: string }[];

  $: descPreview = sanitizeHtml(log.description ?? '');
  $: dlColor = log.due_date ? deadlineColor(log.due_date) : null;
  $: dlText = dlColor ? contrastText(dlColor) : '#fff';

  let hovered = false;
</script>

<article
  class="card" class:closed={log.is_closed}
  on:click={() => dispatch('edit', log)}
  on:mouseenter={() => hovered = true}
  on:mouseleave={() => hovered = false}
  role="button" tabindex="0" on:keydown={e => e.key === 'Enter' && dispatch('edit', log)}
>
  <div class="card-top">
    <div class="title-row">
      <span class="title">{log.title}</span>
      {#if log.is_closed}
        <span class="closed-pill">Closed</span>
      {/if}
    </div>
  </div>

  <div class="type-row">
    {#if log.due_date}
      <span class="deadline" style="background:{dlColor}; color:{dlText};">{log.due_date}</span>
    {/if}
    {#if logType}
      <span class="log-type">{logType.label}</span>
    {/if}
  </div>

  {#if descPreview}
    <div class="desc" class:expanded={hovered} on:click={handleLinkClick}>{@html descPreview}</div>
  {/if}

  <div class="card-footer">
    <div class="badges-row">
      {#each catBadges as { val, type }}
        <Badge label={val.label} catType={type} selected={true} clickable={false} size="sm" />
      {/each}
    </div>
  </div>
</article>

<style>
  .card {
    background: var(--card-bg);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: var(--sp-card-pad, 14px 16px);
    cursor: pointer;
    transition: box-shadow 0.15s, transform 0.12s, border-color 0.15s;
    display: flex;
    flex-direction: column;
    gap: var(--sp-card-gap-inner, 8px);
    min-width: 0;
    align-self: start;
  }
  .card:hover {
    box-shadow: 0 4px 20px rgba(0,0,0,0.1);
    transform: translateY(-2px);
    border-color: var(--accent-muted);
  }
  .card.closed { opacity: 0.5; }

  .card-top { display: flex; flex-direction: column; gap: 2px; }

  .title-row { display: flex; align-items: center; gap: 8px; }
  .title { font-size: 14px; font-weight: 600; color: var(--text); line-height: 1.3; }
  .closed-pill {
    font-size: 10px; font-weight: 600; text-transform: uppercase;
    background: var(--border); color: var(--text-muted);
    border-radius: 999px; padding: 1px 7px;
    flex-shrink: 0;
  }

  .log-type { font-size: 11px; color: var(--text-muted); font-weight: 500; }
  .type-row { display: flex; align-items: center; justify-content: flex-start; gap: 8px; }

  .desc {
    font-size: 12px; color: var(--text-muted); line-height: 1.5;
    margin: 0;
    max-height: 4.5em;
    overflow: hidden;
    display: -webkit-box;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 3;
    transition: max-height 0.3s ease;
  }
  .desc.expanded {
    max-height: 2000px;
    -webkit-line-clamp: unset;
    display: block;
  }
  .desc :global(*) { margin: 0; }
  .desc :global(ul), .desc :global(ol) { padding-left: 18px; margin: 2px 0; }
  .desc :global(li) { margin: 1px 0; }
  .desc :global(table) { border-collapse: collapse; margin: 3px 0; }
  .desc :global(th), .desc :global(td) {
    border: 1px solid var(--border);
    padding: 2px 6px;
    text-align: left;
    vertical-align: top;
  }
  .desc :global(th) { font-weight: 600; }

  .card-footer { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
  .badges-row { display: flex; gap: 4px; flex-wrap: wrap; }

  .deadline {
    font-size: 11px; font-weight: 600;
    border-radius: 6px; padding: 2px 8px;
    white-space: nowrap;
  }
</style>
