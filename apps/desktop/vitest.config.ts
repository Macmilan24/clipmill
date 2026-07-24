import react from '@vitejs/plugin-react';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  test: {
    // Pure helpers do not need a DOM; the screen render tests do. jsdom for
    // everything keeps one config rather than two projects.
    environment: 'jsdom',
    globals: false,
  },
});
