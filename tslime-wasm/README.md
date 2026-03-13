# Tslime Web (WASM)

WebAssembly build of tslime with WebGL2 rendering for browser integration.

## Overview

This package compiles the tslime simulation core to WebAssembly, allowing it to run in browsers with hardware-accelerated rendering via WebGL2.

## Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) (for building)
- Node.js 18+ (for the web demo)

## Building

### 1. Build the WASM module

```bash
cd tslime-wasm
wasm-pack build --target web --out-dir pkg
```

This creates:
- `pkg/tslime_wasm.js` - JavaScript bindings
- `pkg/tslime_wasm_bg.wasm` - WebAssembly binary
- `pkg/tslime_wasm.d.ts` - TypeScript definitions

### 2. Run the demo

```bash
cd examples
# Serve with any static file server, e.g.:
python3 -m http.server 8000
# or
npx serve .
```

Then open http://localhost:8000 in your browser.

## Svelte/Astro Integration

### Installation

1. Copy the built WASM files to your project:
```bash
cp -r tslime-wasm/pkg/* your-project/src/lib/tslime/
```

2. Configure Vite to handle WASM in your `astro.config.mjs`:

```javascript
import { defineConfig } from 'astro/config';
import wasm from 'vite-plugin-wasm';
import topLevelAwait from 'vite-plugin-top-level-await';

export default defineConfig({
  vite: {
    plugins: [wasm(), topLevelAwait()],
  },
});
```

3. Install the plugins:
```bash
npm install vite-plugin-wasm vite-plugin-top-level-await
```

### Usage in Svelte

Create a component (see `examples/svelte-component/Tslime.svelte`):

```svelte
<script>
  import { onMount } from 'svelte';
  import init, { TslimeWasm } from '$lib/tslime/tslime_wasm.js';
  
  let canvas;
  let tslime;
  
  onMount(async () => {
    await init();
    tslime = new TslimeWasm(800, 600, 'tslime-canvas', 42);
    
    // Animation loop
    function loop() {
      tslime.tick();
      requestAnimationFrame(loop);
    }
    loop();
  });
</script>

<canvas id="tslime-canvas" bind:this={canvas} width="800" height="600" />
```

### Usage in Astro

In your `.astro` file:

```astro
---
// Server-side code
---

<TslimeWrapper client:only="svelte" />

<script>
  // Client-side WASM code
  import init, { TslimeWasm } from '../lib/tslime/tslime_wasm.js';
  
  await init();
  const tslime = new TslimeWasm(800, 600, 'tslime-canvas', 42);
</script>
```

**Important:** Use `client:only` directive for WASM components since Node.js cannot execute WASM during SSR.

## API Reference

### `TslimeWasm`

#### Constructor
```javascript
new TslimeWasm(width: number, height: number, canvasId: string, seed: bigint)
```

- `width`, `height` - Simulation grid dimensions
- `canvasId` - ID of the canvas element to render to
- `seed` - Random seed for reproducible simulations

#### Methods

- `step()` - Advance simulation by one frame
- `render()` - Render current state to canvas
- `tick()` - Convenience method: step() + render()
- `start()` - Mark as running (for your own loop management)
- `stop()` - Mark as stopped
- `isRunning()` - Check running state
- `setAgentCount(count: number)` - Reset simulation with different agent count
- `setConfig(sensorAngle, sensorDistance, rotationAngle, stepSize, decay)` - Update simulation parameters

## Performance Tips

1. **Grid size**: Larger grids require more GPU memory. Start with 800x600 and adjust.
2. **Agent count**: Defaults to 1% of grid pixels. Too many agents can slow performance.
3. **WebGL2**: Requires modern browsers. Falls back would need Canvas2D implementation.
4. **Build optimization**: Use `wasm-pack build --release` for production.

## Browser Support

- Chrome 56+ (WebGL2)
- Firefox 51+ (WebGL2)
- Safari 15+ (WebGL2)
- Edge 79+ (WebGL2)

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   JavaScript / Svelte                   │
├─────────────────────────────────────────────────────────┤
│              wasm-bindgen JavaScript glue               │
├─────────────────────────────────────────────────────────┤
│                     WebAssembly                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐   │
│  │  Simulation  │  │    Agents    │  │  Trail Map   │   │
│  │   (tslime)   │  │   (tslime)   │  │   (tslime)   │   │
│  └──────────────┘  └──────────────┘  └──────────────┘   │
│                          │                              │
│                          ▼                              │
│                 ┌──────────────────┐                    │
│                 │  WebGL Renderer  │                    │
│                 │  (tslime-wasm)   │                    │
│                 └──────────────────┘                    │
└─────────────────────────────────────────────────────────┘
                          │
                          ▼
                  ┌──────────────┐
                  │  WebGL2 API  │
                  └──────────────┘
```

## Troubleshooting

### "WebGL2 not supported"
Update your browser or check if hardware acceleration is enabled.

### "Cannot find module"
Make sure you've built the WASM package and the import path is correct.

### Performance issues
- Reduce grid size
- Reduce agent count
- Check browser's GPU acceleration is enabled
- Use release build: `wasm-pack build --release`

## License

MIT - See parent repository for full license.
