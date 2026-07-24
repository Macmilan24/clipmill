import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';

// The shell is only ever served to the Tauri WebView, so the dev server stays
// bound to localhost on a fixed port and never falls back to another one:
// a surprise port would silently break the host's devUrl.
export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  server: {
    host: '127.0.0.1',
    port: 5173,
    strictPort: true,
  },
  build: {
    target: 'es2023',
    // Debugging a local-first app is worth more than a few hundred kilobytes.
    sourcemap: true,
  },
});
