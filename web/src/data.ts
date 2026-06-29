// Curated, structure-driven presets that survive the reduced ANSI pipeline
// (sim-params only — NOT the temporal/glyph showcase presets). Matched by
// name against the wasm preset registry at runtime.
export const CURATED_PRESETS = ['Network', 'Organic', 'Vortex', 'Lightning', 'Tendrils'];
export const CURATED_PALETTES = ['Warm', 'Slime', 'Ocean', 'Mono', 'Heat'];

export const COPY = {
  name: 'tslime',
  tagline: 'A terminal that grows.',
  blurb:
    'A lightweight terminal screensaver that simulates the growth patterns of ' +
    'Physarum polycephalum — slime mold. Organic, algorithmic, alive.',
  github: 'https://github.com/tamirelazar/tslime',
  releases: 'https://github.com/tamirelazar/tslime/releases',
};
