import { defineConfig } from 'vite';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

// Pull the crate version from the workspace Cargo.toml at build time. pages.yml
// rebuilds the site on every release push, so the header version stays correct
// with no manual edit — bump Cargo.toml, cut the release, the page follows.
const cargoToml = readFileSync(fileURLToPath(new URL('../Cargo.toml', import.meta.url)), 'utf8');
const version = cargoToml.match(/^version\s*=\s*"([^"]+)"/m)?.[1] ?? '0.0.0';

export default defineConfig({
  base: '/tslime/',
  define: { __APP_VERSION__: JSON.stringify(version) },
  plugins: [wasm(), topLevelAwait()],
});
