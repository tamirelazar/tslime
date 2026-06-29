import { COPY } from './data';

const CSS = `
  #app{--bg:#0b0d0e;--ink:#c8c0b0;--dim:#8b9398;--warm:#bfa46a;--line:rgba(143,143,85,.30);
    min-height:100vh;display:flex;flex-direction:column;align-items:center;gap:1.4rem;
    padding:0 1rem 4rem}
  #app header{display:flex;justify-content:space-between;align-items:center;width:100%;
    max-width:980px;padding:1.3rem .25rem;border-bottom:1px solid var(--line)}
  #app .wm{font:600 1.1rem "FiraCodeNFMono",monospace;letter-spacing:.02em}
  #app nav a{color:var(--dim);text-decoration:none;margin-left:1.3rem;font-size:.9rem}
  #app nav a:hover{color:var(--warm)}
  #app h1{margin:1.4rem 0 0;font-size:2.1rem;letter-spacing:-.02em;text-align:center}
  #app p{margin:0;max-width:38rem;text-align:center;color:var(--dim);line-height:1.55}
  #app .frame{position:relative;width:min(900px,96vw);aspect-ratio:16/10;background:#1c1916;
    border:1px solid var(--line);border-radius:10px;padding:20px;overflow:hidden;
    box-shadow:0 0 0 1px rgba(143,143,85,.10),0 0 28px rgba(143,143,85,.16),0 18px 50px rgba(0,0,0,.55)}
  #app .frame[data-loading]::after{content:"warming…";position:absolute;inset:0;display:grid;
    place-items:center;color:var(--dim);font:.9rem "FiraCodeNFMono",monospace}
  #app .host{width:100%;height:100%}
  #app .ctl{display:flex;flex-wrap:wrap;gap:.4rem;justify-content:center;align-items:center}
  #app .ctl .lbl{font:600 .72rem "FiraCodeNFMono",monospace;color:var(--dim);
    text-transform:uppercase;letter-spacing:.1em;margin-right:.3rem}
  #app .pill{all:unset;cursor:pointer;font-size:.82rem;padding:.32rem .8rem;border-radius:999px;
    border:1px solid var(--line);color:var(--dim)}
  #app .pill:hover{color:var(--ink);border-color:var(--warm)}
  #app .pill[data-active]{background:var(--warm);color:#1c1916;border-color:var(--warm)}`;

export function buildChrome(app: HTMLElement) {
  const style = document.createElement('style');
  style.textContent = CSS;
  document.head.appendChild(style);
  // Replace the static first-paint contents with the enhanced chrome.
  app.innerHTML = `
    <header>
      <div class="wm">tslime</div>
      <nav><a href="${COPY.github}">GitHub</a><a href="${COPY.releases}">Releases</a></nav>
    </header>
    <h1>${COPY.tagline}</h1>
    <p>${COPY.blurb}</p>
    <div class="frame" data-loading><div class="host" id="host"></div></div>
    <div class="ctl" id="presetBar"><span class="lbl">Preset</span></div>
    <div class="ctl" id="paletteBar"><span class="lbl">Palette</span></div>`;
  return {
    host: app.querySelector<HTMLElement>('#host')!,
    presetBar: app.querySelector<HTMLElement>('#presetBar')!,
    paletteBar: app.querySelector<HTMLElement>('#paletteBar')!,
  };
}
