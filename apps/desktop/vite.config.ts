import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { type Plugin, defineConfig } from 'vite';

/**
 * The UI preview harness has no daemon. With CLIPMILL_CAPTION_ASS naming the
 * `caption_ass` example binary, its captions come from the render's own ASS
 * writer, so the editor preview draws what an export burns in. Development
 * server only; nothing here reaches a build.
 */
function captionAss(): Plugin {
  const binary = process.env.CLIPMILL_CAPTION_ASS;
  return {
    name: 'clipmill-caption-ass',
    apply: 'serve',
    configureServer(server) {
      if (!binary) return;
      server.middlewares.use('/__clipmill/caption-ass', (request, response) => {
        const rate = new URL(request.url ?? '/', 'http://localhost').searchParams.get('rate');
        const child = spawn(binary, rate ? [rate] : []);
        let out = '';
        let problem = '';
        child.stdout.on('data', (chunk: Buffer) => (out += chunk.toString()));
        child.stderr.on('data', (chunk: Buffer) => (problem += chunk.toString()));
        child.on('close', (code) => {
          response.statusCode = code === 0 ? 200 : 422;
          response.setHeader('Content-Type', 'text/plain; charset=utf-8');
          response.end(code === 0 ? out : problem);
        });
        request.pipe(child.stdin);
      });
    },
  };
}

// The shell is only ever served to the Tauri WebView, so the dev server stays
// bound to localhost on a fixed port and never falls back to another one:
// a surprise port would silently break the host's devUrl.
export default defineConfig({
  plugins: [react(), tailwindcss(), captionAss()],
  clearScreen: false,
  // The caption player finds its worker and WebAssembly next to itself with
  // `new URL(…, import.meta.url)`. Pre-bundled, it would look for them in
  // Vite's dependency cache, where they are not, and never start.
  // Its one CommonJS dependency is still converted, as Vite does for any.
  optimizeDeps: { exclude: ['jassub'], include: ['jassub > throughput'] },
  // That worker is a module worker, and it splits into chunks. Vite builds
  // workers as classic scripts by default, which cannot hold chunks, and the
  // production build refuses; every WebView ClipMill runs in loads modules.
  worker: { format: 'es' },
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
