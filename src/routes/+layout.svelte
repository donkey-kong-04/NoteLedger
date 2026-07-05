<script lang="ts">
  import { settings, loadAll, filterDrawerOpen } from '$lib/store';
  import { onMount } from 'svelte';
  import NavTabs from '$lib/components/NavTabs.svelte';

  onMount(() => loadAll());

  $: isDark = $settings.dark_mode;
</script>

<div class="app-shell" class:dark={isDark} class:drawer-open={$filterDrawerOpen}>
  <div class="nav-bar">
    <span class="app-logo">Note Ledger</span>
    <NavTabs />
  </div>
  <div class="page-slot">
    <slot />
  </div>
</div>

<style>
  .app-shell {
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
    --project-header: #ccd2dd;
    --bg:           #f3f4f6;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--surface-2);
    transition: padding-left 0.2s ease;
  }

  /* The filter drawer (fixed, left) pushes the content aside instead of
     covering it; width must match .drawer in FilterPanel.svelte. */
  .app-shell.drawer-open {
    padding-left: min(440px, 92vw);
  }

  /* Logo left, tabs centered — as grid tracks (not absolute positioning) so
     they can't overlap when the open filter drawer narrows the shell. */
  .nav-bar {
    display: grid;
    grid-template-columns: 1fr auto 1fr;
    align-items: center;
    padding: 6px 20px;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .app-logo {
    justify-self: start;
    font-size: 16px; font-weight: 700; color: var(--text);
    letter-spacing: -0.01em; user-select: none;
    white-space: nowrap;
  }

  .page-slot {
    flex: 1; min-height: 0;
    display: flex; flex-direction: column;
  }
  .page-slot > :global(*) { flex: 1; min-height: 0; }

  .app-shell.dark {
    --text:         #f3f4f6;
    --text-muted:   #9ca3af;
    --surface:      #111827;
    --surface-2:    #1f2937;
    --surface-3:    #374151;
    --surface-hover:#4b5563;
    --card-bg:      #1f2937;
    --border:       #374151;
    --project-header: #374151;
    --bg:           #111827;
  }
</style>
