import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { resolve } from 'node:path'

// Fontsource ships each static face as woff2 and woff. Every browser this site
// supports reads woff2, so the woff copies are left out of the build.
const woff2Only = {
  name: 'woff2-only',
  enforce: 'pre',
  transform(code, id) {
    if (!id.includes('/@fontsource/') || !id.endsWith('.css')) return null
    return code.replace(/,\s*url\([^)]*\.woff\)\s*format\('woff'\)/g, '')
  },
}

// seokdok.mkpo.li: a static site. The engine (WebAssembly) and the
// dictionaries load in a worker after the page is up.
export default defineConfig({
  root: import.meta.dirname,
  base: '/',
  plugins: [woff2Only, svelte()],
  worker: { format: 'es' },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    target: 'es2022',
    rollupOptions: {
      input: {
        main: resolve(import.meta.dirname, 'index.html'),
        design: resolve(import.meta.dirname, 'design.html'),
      },
    },
  },
})
