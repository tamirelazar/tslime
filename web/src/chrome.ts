import { COPY } from './data';

// "Instrument" chrome: the live sim is a recessed screen with bordered tags cut
// into its bezel (live / status / readout) and registration ticks at the
// corners. Square, inset, warm-dark field, green the single cold accent.
// Controls = a command line with clickable tokens.
const CSS = `
  #app{--bg:#17120d;--screen:#0b0805;--ink:#c8c0b0;--dim:#8b8378;--acc:#6dbf66;
    --line:rgba(150,140,120,.20);
    font-family:"FiraCodeNFMono",monospace;min-height:100vh;display:flex;
    flex-direction:column;gap:1.6rem;width:min(940px,94vw);margin:0 auto;
    padding:0 0 3rem}
  #app a{color:var(--dim);text-decoration:none}
  #app a:hover{color:var(--acc)}

  #app .top{display:flex;justify-content:space-between;align-items:baseline;
    padding:1.4rem 0 .9rem;border-bottom:1px solid var(--line);gap:1rem;flex-wrap:wrap}
  #app .wm{font-weight:600;font-size:1.05rem;letter-spacing:.02em;color:var(--ink)}
  #app .wm b{color:var(--acc);font-weight:600}
  #app .meta{font-size:.78rem;color:var(--dim);letter-spacing:.04em}
  #app .meta a{margin-left:.9rem}

  #app .lede{display:flex;flex-direction:column;gap:.7rem}
  #app h1{margin:0;font-size:clamp(1.9rem,5vw,2.9rem);font-weight:600;
    letter-spacing:-.01em;line-height:1.05;color:var(--ink)}
  #app h1 i{color:var(--acc);font-style:normal;margin-right:.4rem}
  #app .blurb{margin:0;max-width:58ch;color:var(--dim);font-size:.9rem;line-height:1.6}
  #app .blurb .cta{color:var(--acc);border-bottom:1px solid transparent}
  #app .blurb .cta:hover{color:#a8e29c;border-bottom-color:currentColor}

  /* Recessed screen: square, inner shadow well, bordered bezel tags. */
  #app .screen{position:relative;width:100%;aspect-ratio:16/9;background:var(--screen);
    border:1px solid var(--line);padding:18px;
    box-shadow:inset 0 1px 0 rgba(255,244,228,.05),inset 0 18px 44px -14px rgba(0,0,0,.9),
      inset 0 -10px 30px -16px rgba(0,0,0,.7)}
  #app .viewport{position:absolute;inset:18px;overflow:hidden}
  #app .host{width:100%;height:100%}
  /* registration ticks at the four corners */
  #app .tick{position:absolute;width:9px;height:9px;border:1px solid var(--acc);opacity:.55}
  #app .tick.tl{top:6px;left:6px;border-right:0;border-bottom:0}
  #app .tick.tr{top:6px;right:6px;border-left:0;border-bottom:0}
  #app .tick.bl{bottom:6px;left:6px;border-right:0;border-top:0}
  #app .tick.br{bottom:6px;right:6px;border-left:0;border-top:0}
  /* bordered HUD tags floating just inside the screen, over the sim */
  #app .tag{position:absolute;font-size:.64rem;letter-spacing:.12em;text-transform:uppercase;
    background:#241c14;border:1px solid rgba(150,140,120,.35);padding:.18rem .6rem;color:var(--ink);
    box-shadow:inset 0 1px 0 rgba(255,244,228,.08),0 3px 11px rgba(0,0,0,.75)}
  #app .slabel{top:1.5rem;left:1.5rem}
  #app .status{top:1.5rem;right:1.5rem;color:var(--acc);cursor:pointer}
  #app .status[data-paused]{color:var(--dim)}
  #app .readout{bottom:1.5rem;right:1.5rem;text-transform:none;letter-spacing:.04em}
  #app .readout .leg{color:var(--dim);opacity:.7}
  #app .screen[data-loading] .viewport::after{content:"warming…";position:absolute;
    inset:0;display:grid;place-items:center;color:var(--dim);font-size:.85rem}
  #app .screen[data-error] .viewport{background:center/cover no-repeat url('/tslime/poster.png')}
  #app .screen[data-error] .viewport::after{content:"can't start the sim — here's a still";
    position:absolute;left:0;right:0;bottom:0;padding:.7rem;text-align:center;
    color:var(--ink);font-size:.8rem;background:linear-gradient(transparent,rgba(0,0,0,.7))}

  /* command line: clickable tokens cycle presets/palettes */
  #app .cmd{font-size:clamp(.85rem,2.4vw,1.05rem);color:var(--dim);
    display:flex;flex-wrap:wrap;align-items:center;gap:.5ch}
  #app .cmd .pr{color:var(--acc)}
  #app .cmd .name{color:var(--ink)}
  #app .cmd .tok{color:var(--acc);cursor:pointer;border-bottom:1px dotted var(--acc);
    padding-bottom:1px;transition:color .12s}
  #app .cmd .tok:hover{color:#a8e29c;border-bottom-style:solid}
  #app .cmd .cur{color:var(--acc);animation:blink 1.1s steps(1) infinite;margin-left:.2ch}
  @keyframes blink{50%{opacity:0}}
  #app .hint{font-size:.74rem;color:var(--dim);opacity:.7;line-height:1.5;min-height:1.1em;
    transition:opacity 1.1s ease}
  #app .hint[data-faded]{opacity:0}

  /* Phones: the 16/9 screen is too short — give the sim a taller viewport. */
  @media (max-width:640px){#app .screen{aspect-ratio:4/5}}`;

export interface ChromeHandles {
  host: HTMLElement;
  screen: HTMLElement;
  tokPreset: HTMLElement;
  tokPalette: HTMLElement;
  status: HTMLElement;
  readout: HTMLElement;
  hint: HTMLElement;
  cursor: HTMLElement;
  chevron: HTMLElement;
}

export function buildChrome(app: HTMLElement): ChromeHandles {
  const style = document.createElement('style');
  style.textContent = CSS;
  document.head.appendChild(style);
  app.innerHTML = `
    <div class="top">
      <div class="wm">~ $ <b>tslime</b></div>
      <div class="meta">v${COPY.version}<a href="${COPY.github}">github</a><a href="${COPY.releases}">releases</a></div>
    </div>
    <div class="lede">
      <h1><i id="chev">❯</i>${COPY.tagline.replace(/\.$/, '')}</h1>
      <p class="blurb">${COPY.blurb} <a class="cta" href="${COPY.releases}">${COPY.cta} ↗</a></p>
    </div>
    <div class="screen" data-loading>
      <span class="tick tl"></span><span class="tick tr"></span>
      <span class="tick bl"></span><span class="tick br"></span>
      <span class="tag slabel">live</span>
      <span class="tag status" id="status">● running</span>
      <div class="viewport"><div class="host" id="host"></div></div>
      <span class="tag readout" id="readout">—</span>
    </div>
    <div class="cmd" id="cmd">
      <span class="pr">$</span> <span class="name">tslime</span>
      <span class="flag">--preset</span> <span class="tok" id="tokPreset" role="button" tabindex="0">network</span>
      <span class="flag">--palette</span> <span class="tok" id="tokPalette" role="button" tabindex="0">slime</span>
      <span class="cur" id="cursor">█</span>
    </div>
    <div class="hint" id="hint">click a value to cycle</div>`;
  const q = <T extends HTMLElement>(s: string) => app.querySelector<T>(s)!;
  return {
    host: q('#host'),
    screen: q('.screen'),
    tokPreset: q('#tokPreset'),
    tokPalette: q('#tokPalette'),
    status: q('#status'),
    readout: q('#readout'),
    hint: q('#hint'),
    cursor: q('#cursor'),
    chevron: q('#chev'),
  };
}
