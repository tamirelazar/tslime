// Curated, structure-driven presets that survive the reduced ANSI pipeline
// (sim-params only — NOT the temporal/glyph showcase presets). Matched by
// name against the wasm preset registry at runtime.
export const CURATED_PRESETS = ['Network', 'Organic', 'Vortex', 'Lightning', 'Tendrils'];
export const CURATED_PALETTES = ['Slime', 'Warm', 'Ocean', 'Mono', 'Cosmic'];

// Injected from Cargo.toml at build time (see vite.config.ts).
declare const __APP_VERSION__: string;

export const COPY = {
  name: 'tslime',
  version: __APP_VERSION__,
  tagline: 'A terminal screensaver that will grow on you.',
  blurb: 'A limited WASM demo.',
  cta: 'install the real thing',
  github: 'https://github.com/tamirelazar/tslime',
  releases: 'https://github.com/tamirelazar/tslime/releases',
};
