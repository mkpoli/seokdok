import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// seokdok.mkpo.li: a static site. The engine (WebAssembly) and the
// dictionaries load in a worker after the page is up.
export default defineConfig({
  root: import.meta.dirname,
  base: '/',
  plugins: [svelte()],
  worker: { format: 'es' },
  build: { outDir: 'dist', emptyOutDir: true, target: 'es2022' },
})
