import { CURATED_PRESETS, CURATED_PALETTES } from './data';
import type { TermController } from './terminal';
import type { ChromeHandles } from './chrome';

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

  // The cursor + hint guide first use; they retire once the user cycles a value.
  let guided = true;
  const onCycle = () => {
    readout();
    if (guided) {
      guided = false;
      h.hint.setAttribute('data-done', '');
      h.cursor.style.display = 'none';
    }
  };
  cycler(h.tokPreset, CURATED_PRESETS, (n) => c.setPresetByName(n), onCycle);
  cycler(h.tokPalette, CURATED_PALETTES, (n) => c.setPaletteByName(n), onCycle);
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
