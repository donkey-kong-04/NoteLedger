import { defineConfig } from 'vitest/config';

// Standalone config (no Svelte plugin) — the unit tests target pure TS modules.
export default defineConfig({
  test: {
    include: ['src/**/*.test.ts'],
    environment: 'node',
  },
});
