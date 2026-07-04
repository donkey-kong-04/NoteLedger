<script lang="ts">
  import { page } from '$app/stores';

  // Central list of app views — add new pages here and they appear in every top bar.
  const tabs = [
    { label: 'Home', href: '/' },
    { label: 'Table view', href: '/table' },
    { label: 'Templates', href: '/templates' },
    { label: 'Settings', href: '/settings' },
  ];

  $: current = $page.url.pathname;
</script>

<nav class="view-tabs" aria-label="Main views">
  {#each tabs as tab (tab.href)}
    <a href={tab.href} class="tab" class:active={current === tab.href} aria-current={current === tab.href ? 'page' : undefined}>
      {tab.label}
    </a>
  {/each}
</nav>

<style>
  .view-tabs {
    display: inline-flex; align-items: center;
    background: var(--surface-3);
    border: 1px solid var(--border);
    border-radius: 10px;
    padding: 3px;
    gap: 3px;
  }
  .tab {
    font-size: 13px; font-weight: 600;
    color: var(--text-muted);
    text-decoration: none;
    padding: 5px 14px;
    border-radius: 7px;
    transition: background 0.15s, color 0.15s;
    white-space: nowrap;
  }
  .tab:hover { color: var(--text); background: var(--surface-hover); }
  .tab.active {
    background: var(--accent);
    color: #fff;
    box-shadow: 0 1px 3px rgba(0,0,0,0.2);
  }
  .tab.active:hover { color: #fff; background: var(--accent); }
</style>
