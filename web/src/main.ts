import { buildChrome } from './chrome';
import { mountTerminal } from './terminal';

async function main() {
  const app = document.getElementById('app')!;
  const { host } = buildChrome(app);
  const frame = host.parentElement!;
  const controller = await mountTerminal(host);
  frame.removeAttribute('data-loading');
  (window as any).__tslime = controller; // wired to pills in Task 10
}
main();
