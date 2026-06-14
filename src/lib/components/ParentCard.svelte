<script lang="ts">
  import LogCard from './LogCard.svelte';
  import { createEventDispatcher } from 'svelte';
  import type { Log, PicklistValue } from '../types';

  export let parent: Log;
  export let children: Log[];
  export let logTypes: PicklistValue[];
  export let cat1Vals: PicklistValue[];
  export let cat2Vals: PicklistValue[];
  export let cat3Vals: PicklistValue[];
  export let cat4Vals: PicklistValue[];

  const dispatch = createEventDispatcher();
</script>

<div class="parent-card">
  <div class="parent-title-col" role="button" tabindex="0"
    on:click={() => dispatch('edit', parent)}
    on:keydown={e => e.key === 'Enter' && dispatch('edit', parent)}
  >
    <span class="parent-title">{parent.title}</span>
  </div>
  <div class="children">
    {#each children as child (child.id)}
      <LogCard
        log={child}
        {logTypes} {cat1Vals} {cat2Vals} {cat3Vals} {cat4Vals}
        on:edit
      />
    {/each}
  </div>
</div>

<style>
  .parent-card {
    display: flex;
    gap: 0;
    border: 1.5px solid var(--border);
    border-radius: 14px;
    overflow: hidden;
    background: var(--surface-2);
  }

  .parent-title-col {
    width: 32px;
    min-width: 32px;
    background: var(--surface-3);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    border-right: 1px solid var(--border);
    transition: background 0.15s;
  }
  .parent-title-col:hover { background: var(--surface-hover); }

  .parent-title {
    writing-mode: vertical-rl;
    text-orientation: mixed;
    transform: rotate(180deg);
    font-size: 12px;
    font-weight: 700;
    color: var(--text-muted);
    letter-spacing: 0.05em;
    white-space: nowrap;
    max-height: 200px;
    overflow: hidden;
    text-overflow: ellipsis;
    user-select: none;
  }

  .children {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px;
    min-width: 0;
  }
</style>
