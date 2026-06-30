import { CURATED_PRESETS, CURATED_PALETTES } from './data';
import type { TermController } from './terminal';
import type { ChromeHandles } from './chrome';

// Chevron accent per palette: a lifted, slime-parallel sample of each palette's
// hue (same lightness/chroma profile the Slime green carries as the UI accent).
const PALETTE_ACCENT: Record<string, string> = {
  Slime: '#6dbf66',
  Warm: '#d2ab5f',
  Ocean: '#79aff7',
  Mono: '#b0adaa',
  Heat: '#f18c64',
};

// One-line character note per preset, shown after the first preset cycle.
const PRESET_INFO: Record<string, string> = {
  Network: 'dense, interconnected veins with rapid branching',
  Organic: 'balanced, natural-looking growth',
  Vortex: 'rotational currents that coil into spirals',
  Lightning: 'fast dendritic branching, like lightning',
  Tendrils: 'long arms reaching across the field',
};

// A clickable command-line token that cycles a curated list and live-swaps.
function cycler(el: HTMLElement, names: string[], onPick: (n: string) => void, onChange: () => void) {
  let i = 0;
  el.textContent = names[0];
  const next = () => {
    i = (i + 1) % names.length;
    el.textContent = names[i];
    onPick(names[i]);
    onChange();
  };
  el.addEventListener('click', next);
  el.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); next(); }
  });
}

export function wireControls(h: ChromeHandles, c: TermController, agents: number) {
  const agentLabel = agents >= 1000 ? `${Math.round(agents / 1000)}k` : `${agents}`;
  const readout = () => {
    h.readout.innerHTML =
      `${h.tokPreset.textContent} · ${h.tokPalette.textContent} · ${agentLabel}` +
      ` <span class="leg">· space pauses</span>`;
  };

  // The cursor + "click a value to cycle" guide retire on the first cycle of
  // either token. The preset info line only appears once a preset is cycled — a
  // palette-first interaction leaves the line empty until then.
  let guided = true;
  const retireGuide = () => {
    if (!guided) return;
    guided = false;
    h.cursor.style.display = 'none';
    h.hint.textContent = '';
  };

  // Show the preset note, then let it fade out gently after a long-ish dwell.
  // A fresh cycle cancels the pending fade and brings the line back.
  let fadeTimer: number | undefined;
  const showPresetInfo = (text: string) => {
    if (fadeTimer) clearTimeout(fadeTimer);
    h.hint.removeAttribute('data-faded');
    h.hint.textContent = text;
    fadeTimer = window.setTimeout(() => h.hint.setAttribute('data-faded', ''), 7000);
  };
  cycler(h.tokPreset, CURATED_PRESETS, (n) => c.setPresetByName(n), () => {
    retireGuide();
    showPresetInfo(PRESET_INFO[h.tokPreset.textContent ?? ''] ?? '');
    readout();
  });
  cycler(h.tokPalette, CURATED_PALETTES, (n) => {
    c.setPaletteByName(n);
    const col = PALETTE_ACCENT[n] ?? 'var(--acc)';
    h.chevron.style.color = col;
    h.tokPalette.style.color = col;
    h.tokPalette.style.borderBottomColor = col;
  }, () => {
    retireGuide();
    readout();
  });
  readout();

  // Pause/resume: space (global) or clicking the status tag.
  let playing = true;
  const setPlaying = (v: boolean) => {
    playing = v;
    playing ? c.resume() : c.pause();
    h.status.textContent = playing ? '● running' : '❚❚ paused';
    h.status.toggleAttribute('data-paused', !playing);
  };
  h.status.addEventListener('click', () => setPlaying(!playing));
  window.addEventListener('keydown', (e) => {
    if (e.code === 'Space' && !/^(INPUT|TEXTAREA)$/.test((e.target as HTMLElement)?.tagName)) {
      e.preventDefault();
      setPlaying(!playing);
    }
  });

  return { setPlaying };
}
