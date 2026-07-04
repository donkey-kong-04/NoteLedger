<script lang="ts">
  import { settings, loadAll } from '$lib/store';
  import { onMount } from 'svelte';
  import NavTabs from '$lib/components/NavTabs.svelte';

  onMount(() => loadAll());

  $: isDark = $settings.dark_mode;
</script>

<div class="app-shell" class:dark={isDark}>
  <div class="nav-bar">
    <svg class="app-logo" width="36" height="40" viewBox="0 0 36 40" aria-label="Note Ledger">
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
    --bg:           #f3f4f6;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--surface-2);
  }

  .nav-bar {
    position: relative;
    display: flex; justify-content: center; align-items: center;
    padding: 6px 0;
    background: var(--surface);
    border-bottom: 1px solid var(--border);
    flex-shrink: 0;
  }
  .app-logo { position: absolute; left: 20px; }

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
    --bg:           #111827;
  }
</style>
