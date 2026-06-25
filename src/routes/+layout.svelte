<script lang="ts">
  import { settings, loadAll } from '$lib/store';
  import { onMount } from 'svelte';

  onMount(() => loadAll());

  $: isDark = $settings.dark_mode;
</script>

<div class="app-shell" class:dark={isDark}>
  <slot />
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
    display: contents;
  }

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
