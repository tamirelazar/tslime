import { buildChrome } from './chrome';
import { mountTerminal } from './terminal';
import { wireControls } from './controls';

async function main() {
  const app = document.getElementById('app')!;
  const { host, presetBar, paletteBar } = buildChrome(app);
  const frame = host.parentElement!;
  const controller = await mountTerminal(host);
  frame.removeAttribute('data-loading');
  wireControls(presetBar, paletteBar, controller);
}
main();
