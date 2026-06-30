import { buildChrome } from './chrome';
import { mountTerminal } from './terminal';
import { wireControls } from './controls';

async function main() {
  const app = document.getElementById('app')!;
  const h = buildChrome(app);

  // Mobile / small viewport: lighter agent population.
  const mobile = window.matchMedia('(max-width: 640px)').matches;
  const agents = mobile ? 12000 : 50000;
  const reduce = window.matchMedia('(prefers-reduced-motion: reduce)').matches;

  let controller;
  try {
    controller = await mountTerminal(h.host, { agents: mobile ? agents : undefined });
  } catch (err) {
    console.error('tslime wasm failed to start', err);
    h.screen.removeAttribute('data-loading');
    h.screen.setAttribute('data-error', '');
    return; // poster fallback shown via CSS
  }
  h.screen.removeAttribute('data-loading');

  const ctl = wireControls(h, controller, agents);

  // Honor reduced-motion by starting paused.
  if (reduce) ctl.setPlaying(false);
}
main();
