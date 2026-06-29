import { buildChrome } from './chrome';
import { mountTerminal } from './terminal';
import { wireControls } from './controls';

async function main() {
  const app = document.getElementById('app')!;
  const { host, presetBar, paletteBar } = buildChrome(app);
  const frame = host.parentElement!;

  // Mobile / small viewport: lighter agent population.
  const agents = window.matchMedia('(max-width: 640px)').matches ? 12000 : undefined;
  const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  let controller;
  try {
    controller = await mountTerminal(host, { agents });
  } catch (err) {
    console.error('tslime wasm failed to start', err);
    frame.removeAttribute('data-loading');
    frame.setAttribute('data-error', '');
    return; // poster fallback shown via CSS; pills omitted
  }
  frame.removeAttribute('data-loading');
  wireControls(presetBar, paletteBar, controller);

  // Pause/Play control; honor reduced-motion by starting paused.
  const toggle = document.createElement('button');
  toggle.className = 'toggle';
  let playing = !reduce;
  if (reduce) controller.pause();
  toggle.textContent = playing ? 'Pause' : 'Play';
  toggle.addEventListener('click', () => {
    playing = !playing;
    playing ? controller!.resume() : controller!.pause();
    toggle.textContent = playing ? 'Pause' : 'Play';
  });
  paletteBar.after(toggle);
}
main();
