<script lang="ts">
  import { get } from 'svelte/store';
  import { onDestroy } from 'svelte';
  import CategoryFilter from './CategoryFilter.svelte';
  import { picklists, projects, settings, saveSettings, showClosed, selCat1, selCat2, selCat3, selCat4, selProject, selLogType, filterDrawerOpen } from '../store';
  import { sortedProjectOptions, CAT_COLORS } from '../types';
  import type { PicklistValue } from '../types';

  // Open state is shared via the store so +layout.svelte can shift the page
  // content aside. Reset on destroy so navigating to a page without a
  // FilterPanel doesn't leave the content shifted next to nothing.
  onDestroy(() => filterDrawerOpen.set(false));

  // Category labels are editable from the drawer (click a section label);
  // persisted to settings like the old sidebar did.
  let cat1Label = '';
  let cat2Label = '';
  let cat3Label = '';
  let cat4Label = '';
  $: { cat1Label = $settings.category1_label; cat2Label = $settings.category2_label; cat3Label = $settings.category3_label; cat4Label = $settings.category4_label; }

  async function persistLabelChange() {
    const s = get(settings);
    await saveSettings({ ...s, category1_label: cat1Label, category2_label: cat2Label, category3_label: cat3Label, category4_label: cat4Label });
  }

  $: nonTemplateProjects = $projects.filter(p => !p.is_template);
  $: logTypes = $picklists.filter(v => v.picklist_type === 'log_type');
  const byLabel = (a: PicklistValue, b: PicklistValue) => a.label.localeCompare(b.label);
  $: cat1Vals = $picklists.filter(v => v.picklist_type === 'category_1').sort(byLabel);
  $: cat2Vals = $picklists.filter(v => v.picklist_type === 'category_2').sort(byLabel);
  $: cat3Vals = $picklists.filter(v => v.picklist_type === 'category_3').sort(byLabel);
  $: cat4Vals = $picklists.filter(v => v.picklist_type === 'category_4').sort(byLabel);

  // ── Project lookup (same behavior as the old top-bar lookup) ─────────────
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
    // small delay so click on option fires first
    setTimeout(() => { lookupOpen = false; lookupSearch = ''; }, 150);
  }

  function clearAll() {
    $selCat1 = []; $selCat2 = []; $selCat3 = []; $selCat4 = [];
    $selProject = null;
    $selLogType = null;
    $showClosed = false;
  }

  // ── Active filter chips (always visible in the bar, each dismissible) ────
  type Chip = { key: string; label: string; color?: { hex: string; rgb: string }; clear: () => void };

  $: catChipDefs = [
    { sel: $selCat1, vals: cat1Vals, type: 'category_1', store: selCat1 },
    { sel: $selCat2, vals: cat2Vals, type: 'category_2', store: selCat2 },
    { sel: $selCat3, vals: cat3Vals, type: 'category_3', store: selCat3 },
    { sel: $selCat4, vals: cat4Vals, type: 'category_4', store: selCat4 },
  ];

  $: chips = [
    ...($selProject != null ? [{
      key: 'project',
      label: selectedProjectLabel || 'Project',
      clear: () => selProject.set(null),
    }] : []),
    ...($selLogType != null ? [{
      key: 'logtype',
      label: logTypes.find(t => t.id === $selLogType)?.label ?? 'Type',
      clear: () => selLogType.set(null),
    }] : []),
    ...catChipDefs.flatMap(d => d.sel.map(id => ({
      key: `${d.type}-${id}`,
      label: d.vals.find(v => v.id === id)?.label ?? '…',
      color: CAT_COLORS[d.type],
      clear: () => d.store.update(a => a.filter(x => x !== id)),
    }))),
    ...($showClosed ? [{
      key: 'closed',
      label: 'Closed shown',
      clear: () => showClosed.set(false),
    }] : []),
  ] as Chip[];

  $: activeCount = chips.length;

  function onWindowKeydown(e: KeyboardEvent) {
    if (e.key !== 'Escape' || !$filterDrawerOpen) return;
    const t = e.target as HTMLElement | null;
    // Inputs handle Escape themselves (lookup, badge search/edit).
    if (t && (t.tagName === 'INPUT' || t.tagName === 'TEXTAREA' || t.isContentEditable)) return;
    $filterDrawerOpen = false;
  }

  let drawerEl: HTMLElement;
  let controlsEl: HTMLElement;

  // Close when clicking outside the drawer (and outside the funnel/chips bar).
  // mousedown, not click: handlers inside the drawer can remove their element
  // from the DOM (chip ×, lookup options), which would make a click-time
  // contains() check fail and close the drawer on an inside interaction.
  function onWindowMousedown(e: MouseEvent) {
    if (!$filterDrawerOpen) return;
    const t = e.target as Node;
    if (drawerEl?.contains(t) || controlsEl?.contains(t)) return;
    $filterDrawerOpen = false;
  }
</script>

<svelte:window on:keydown={onWindowKeydown} on:mousedown={onWindowMousedown} />

<div class="filter-controls" bind:this={controlsEl}>
  <button
    class="funnel-btn" class:engaged={activeCount > 0 || $filterDrawerOpen}
    on:click={() => $filterDrawerOpen = !$filterDrawerOpen}
    title="Filters" aria-label="Filters" aria-expanded={$filterDrawerOpen}
  >
    <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
      <path d="M1.5 2.5h13l-5 6v4.5l-3 1.5V8.5l-5-6z"/>
    </svg>
    {#if activeCount > 0}
      <span class="count-badge">{activeCount}</span>
    {/if}
  </button>

  {#each chips as chip (chip.key)}
    <span class="chip" style={chip.color ? `background: rgba(${chip.color.rgb}, 0.14); color: ${chip.color.hex};` : ''}>
      {chip.label}
      <button class="chip-x" on:click={chip.clear} title="Remove filter" aria-label="Remove filter {chip.label}">×</button>
    </span>
  {/each}
</div>

{#if $filterDrawerOpen}
  <aside class="drawer" aria-label="Filters" bind:this={drawerEl}>
    <div class="drawer-header">
      <h2>Filters</h2>
      <div class="drawer-actions">
        {#if activeCount > 0}
          <button class="btn-clear" on:click={clearAll}>✕ Clear all</button>
        {/if}
        <button class="icon-btn" on:click={() => $filterDrawerOpen = false} aria-label="Close">✕</button>
      </div>
    </div>

    <div class="drawer-body">
      <section>
        <div class="section-title">Project filters</div>
        <div class="project-row">
          {#if nonTemplateProjects.length > 0}
            <div class="project-lookup" class:active={lookupOpen}>
              <input
                bind:this={lookupInput}
                class="project-lookup-input"
                type="text"
                placeholder={$selProject != null ? selectedProjectLabel : 'Search project…'}
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
          <label class="toggle-wrap" title="Show closed items">
            <span class="toggle-switch" class:on={$showClosed} on:click={() => $showClosed = !$showClosed} role="switch" aria-checked={$showClosed} tabindex="0" on:keydown={e => e.key === ' ' && ($showClosed = !$showClosed)}>
              <span class="toggle-thumb"></span>
            </span>
            <span class="toggle-label">Show closed</span>
          </label>
        </div>
      </section>

      <section>
        <div class="section-title">Log filters</div>
        {#if logTypes.length > 0}
          <select class="logtype-filter" class:active={$selLogType !== null} bind:value={$selLogType} title="Filter by log type">
            <option value={null}>All types</option>
            {#each logTypes as t (t.id)}
              <option value={t.id}>{t.label}</option>
            {/each}
          </select>
        {:else}
          <p class="hint">No log types defined yet.</p>
        {/if}
      </section>

      <section>
        <div class="section-title">Category filters</div>
        <div class="cat-grid">
          <div class="cat-cell">
            <CategoryFilter catType="category_1" bind:label={cat1Label} values={cat1Vals} bind:selected={$selCat1} layout="vertical" on:labelChange={persistLabelChange} />
          </div>
          <div class="cat-cell">
            <CategoryFilter catType="category_2" bind:label={cat2Label} values={cat2Vals} bind:selected={$selCat2} layout="vertical" on:labelChange={persistLabelChange} />
          </div>
          <div class="cat-cell">
            <CategoryFilter catType="category_3" bind:label={cat3Label} values={cat3Vals} bind:selected={$selCat3} layout="vertical" on:labelChange={persistLabelChange} />
          </div>
          <div class="cat-cell">
            <CategoryFilter catType="category_4" bind:label={cat4Label} values={cat4Vals} bind:selected={$selCat4} layout="vertical" on:labelChange={persistLabelChange} />
          </div>
        </div>
      </section>
    </div>
  </aside>
{/if}

<style>
  .filter-controls {
    display: flex; align-items: center; gap: 8px; flex-wrap: wrap;
    min-width: 0;
  }

  .funnel-btn {
    position: relative;
    width: 32px; height: 32px; flex-shrink: 0;
    display: flex; align-items: center; justify-content: center;
    background: var(--surface-2); color: var(--text-muted);
    border: 1px solid var(--border); border-radius: 8px;
    cursor: pointer; transition: color 0.15s, border-color 0.15s;
  }
  .funnel-btn:hover { color: var(--text); border-color: var(--text-muted); }
  .funnel-btn.engaged { color: var(--accent); border-color: var(--accent); }

  .count-badge {
    position: absolute; top: -6px; right: -6px;
    min-width: 16px; height: 16px; padding: 0 4px;
    display: flex; align-items: center; justify-content: center;
    background: var(--accent); color: #fff;
    font-size: 10px; font-weight: 700;
    border-radius: 999px;
  }

  .chip {
    display: inline-flex; align-items: center; gap: 4px;
    background: rgba(99,102,241,0.12); color: var(--accent);
    font-size: 12px; font-weight: 600;
    border-radius: 999px; padding: 3px 6px 3px 10px;
    white-space: nowrap;
  }
  .chip-x {
    background: none; border: none; cursor: pointer;
    color: inherit; font-size: 13px; line-height: 1;
    padding: 0 2px; opacity: 0.7;
  }
  .chip-x:hover { opacity: 1; }

  /* Fixed to the viewport's left edge; the app shell in +layout.svelte adds
     matching padding-left while open, so content sits beside, not under. */
  .drawer {
    position: fixed; top: 0; left: 0; bottom: 0;
    width: 440px; max-width: 92vw;
    background: var(--surface);
    border-right: 1px solid var(--border);
    z-index: 90;
    display: flex; flex-direction: column;
    animation: slideInLeft 0.2s ease;
  }
  @keyframes slideInLeft {
    from { transform: translateX(-100%); opacity: 0; }
    to { transform: translateX(0); opacity: 1; }
  }

  .drawer-header {
    display: flex; align-items: center; justify-content: space-between;
    padding: 16px 24px; border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .drawer-header h2 { margin: 0; font-size: 17px; font-weight: 700; color: var(--text); }
  .drawer-actions { display: flex; align-items: center; gap: 8px; }

  .btn-clear {
    background: none; color: #ef4444;
    border: 1px solid #ef4444; border-radius: 8px;
    padding: 5px 10px; font-size: 12px; font-weight: 600;
    cursor: pointer; font-family: inherit; transition: background 0.15s;
  }
  .btn-clear:hover { background: rgba(239,68,68,0.1); }

  .icon-btn {
    width: 30px; height: 30px; border: none; background: none;
    color: var(--text-muted); cursor: pointer; border-radius: 6px; font-size: 16px;
    display: flex; align-items: center; justify-content: center; transition: background 0.15s;
  }
  .icon-btn:hover { background: var(--surface-2); color: var(--text); }

  .drawer-body {
    flex: 1; overflow-y: auto;
    padding: 20px 24px;
    display: flex; flex-direction: column; gap: 22px;
  }

  .section-title {
    font-size: 11px; font-weight: 700;
    text-transform: uppercase; letter-spacing: 0.08em;
    color: var(--text-muted);
    margin-bottom: 10px;
  }

  .project-row { display: flex; align-items: center; gap: 14px; }

  .hint { margin: 0; font-size: 12px; color: var(--text-muted); }

  .cat-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 20px 22px;
  }
  .cat-cell { min-width: 0; }

  /* ── Controls (same look as the old top-bar versions) ── */
  .toggle-wrap {
    display: flex; align-items: center; gap: 7px; cursor: pointer; user-select: none;
    flex-shrink: 0;
  }
  .toggle-label { font-size: 13px; font-weight: 600; color: var(--text-muted); white-space: nowrap; }
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

  .project-lookup { position: relative; display: flex; align-items: center; flex: 1; min-width: 0; }
  .project-lookup-input {
    background: var(--surface-2); color: var(--text);
    border: 1px solid var(--border); border-radius: 8px;
    padding: 6px 10px; font-size: 13px; font-family: inherit;
    outline: none; width: 100%;
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
    outline: none; cursor: pointer; width: 50%;
    transition: border-color 0.15s;
  }
  .logtype-filter:focus { border-color: var(--accent); }
  .logtype-filter.active { border-color: var(--accent); color: var(--accent); font-weight: 600; }
</style>
