import { CURATED_PRESETS, CURATED_PALETTES } from './data';
import type { TermController } from './terminal';

function bar(
  container: HTMLElement,
  names: string[],
  onPick: (name: string) => void,
) {
  names.forEach((name, i) => {
    const b = document.createElement('button');
    b.className = 'pill';
    b.textContent = name;
    if (i === 0) b.setAttribute('data-active', '');
    b.addEventListener('click', () => {
      container.querySelectorAll('.pill').forEach((p) => p.removeAttribute('data-active'));
      b.setAttribute('data-active', '');
      onPick(name);
    });
    container.appendChild(b);
  });
}

export function wireControls(
  presetBar: HTMLElement,
  paletteBar: HTMLElement,
  c: TermController,
) {
  bar(presetBar, CURATED_PRESETS, (n) => c.setPresetByName(n));
  bar(paletteBar, CURATED_PALETTES, (n) => c.setPaletteByName(n));
}
