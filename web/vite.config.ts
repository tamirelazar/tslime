import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

// Served from https://tamirelazar.github.io/tslime/, so assets resolve under
// the `/tslime/` sub-path. `wasm` + `topLevelAwait` mirror the personal site's
// Vite setup so the wasm-pack module loads the same way in both.
export default defineConfig({
  base: '/tslime/',
  plugins: [svelte(), wasm(), topLevelAwait()],
});
