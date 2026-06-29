import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebglAddon } from '@xterm/addon-webgl';
import '@xterm/xterm/css/xterm.css';
import init, { TslimeWasm } from './lib/tslime/tslime_wasm.js';

const SIM_W = 400, SIM_H = 200, SEED = 1, TARGET_DENSITY = 70;

export interface TermController {
  setPresetByName(name: string): void;
  setPaletteByName(name: string): void;
  pause(): void;
  resume(): void;
}

export async function mountTerminal(
  host: HTMLElement,
  opts: { agents?: number } = {},
): Promise<TermController> {
  await init();
  await document.fonts.load('13px "FiraCodeNFMono"').catch(() => {});

  const term = new Terminal({
    fontFamily: '"FiraCodeNFMono", monospace',
    fontSize: 13, lineHeight: 1,
    theme: { background: 'rgba(0,0,0,0)', foreground: '#c8c0b0',
      cursor: 'rgba(0,0,0,0)', selectionBackground: 'transparent' },
    cursorBlink: false, disableStdin: true, scrollback: 0,
    allowTransparency: true, convertEol: false, customGlyphs: true,
  });
  const fit = new FitAddon();
  term.loadAddon(fit);
  term.open(host);
  if (!navigator.webdriver) {
    try { term.loadAddon(new WebglAddon()); } catch { /* DOM fallback */ }
  }

  const sim = new TslimeWasm(SIM_W, SIM_H, '', SEED);
  if (opts.agents) sim.set_agent_count(opts.agents);

  // Build name -> id maps from the wasm registries (avoids index drift).
  const presetId = new Map<string, number>();
  for (let i = 0; i < sim.preset_count(); i++) presetId.set(sim.preset_name(i), i);
  const paletteId = new Map<string, number>();
  for (let i = 0; i < sim.palette_count(); i++) paletteId.set(sim.palette_name(i), i);

  // Warm to a mature network in chunks so the tab never freezes.
  await new Promise<void>((resolve) => {
    let done = 0;
    const chunk = () => {
      for (let i = 0; i < 50 && done < 300; i++, done++) sim.step();
      if (done < 300) requestAnimationFrame(chunk);
      else resolve();
    };
    chunk();
  });

  let cols = 0, rows = 0, paused = false;
  const refit = () => {
    const w = host.clientWidth, h = host.clientHeight;
    if (!w || !h) return;
    term.options.fontSize = Math.max(4, Math.floor(Math.min(w, h) / TARGET_DENSITY));
    fit.fit();
    cols = term.cols; rows = term.rows;
  };
  refit();
  new ResizeObserver(refit).observe(host);

  const frame = () => {
    if (!paused) {
      sim.step();
      if (cols > 0 && rows > 0) term.write(sim.render_ansi_frame(cols, rows));
    }
    requestAnimationFrame(frame);
  };
  requestAnimationFrame(frame);

  return {
    setPresetByName(name) { const id = presetId.get(name); if (id !== undefined) sim.set_preset(id); },
    setPaletteByName(name) { const id = paletteId.get(name); if (id !== undefined) sim.set_palette(id); },
    pause() { paused = true; },
    resume() { paused = false; },
  };
}
