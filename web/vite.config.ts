/// <reference types="vitest/config" />
import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

export default defineConfig({
  // Served from the root of delta.nandish.dev, so no /repo-name/ prefix.
  base: '/',
  plugins: [react(), tailwindcss()],
  worker: { format: 'es' },
  // The real budget is gzipped first load ≤ 350 KB (docs/plan/05-PLAYGROUND-UX.md §9);
  // React + CodeMirror alone are ~600 KB minified, so the default 500 KB warning is noise.
  build: { chunkSizeWarningLimit: 800 },
  test: { include: ['src/**/*.test.ts'] },
});
