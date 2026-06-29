<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  // Built by `pnpm run wasm` (wasm-pack) into ./tslime/ — gitignored.
  import init, { TslimeWasm } from './tslime/tslime_wasm.js';

  let { width = 800, height = 600 }: { width?: number; height?: number } = $props();

  let canvas: HTMLCanvasElement;
  let sim: TslimeWasm | null = null;
  let frame = 0;
  let running = $state(false);
  let ready = $state(false);

  const randomSeed = () => Math.floor(Math.random() * 1_000_000);

  function start() {
    if (!sim || running) return;
    running = true;
    sim.start();
    const loop = () => {
      if (!sim || !running) return;
      sim.tick();
      frame = requestAnimationFrame(loop);
    };
    loop();
  }

  function stop() {
    running = false;
    if (frame) cancelAnimationFrame(frame);
    sim?.stop();
  }

  function restart() {
    stop();
    sim = new TslimeWasm(width, height, 'tslime-canvas', randomSeed());
    start();
  }

  onMount(async () => {
    await init();
    canvas.width = width;
    canvas.height = height;
    sim = new TslimeWasm(width, height, 'tslime-canvas', randomSeed());
    ready = true;
    start();
  });

  onDestroy(stop);
</script>

<div class="tslime">
  <canvas id="tslime-canvas" bind:this={canvas}></canvas>
  <div class="controls">
    <button onclick={start} disabled={!ready || running}>Start</button>
    <button onclick={stop} disabled={!ready || !running}>Stop</button>
    <button onclick={restart} disabled={!ready}>New seed</button>
  </div>
</div>

<style>
  .tslime {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    width: 100%;
  }

  canvas {
    width: 100%;
    max-width: 800px;
    height: auto;
    aspect-ratio: 4 / 3;
    background: #000;
    border: 1px solid #20272b;
    border-radius: 10px;
  }

  .controls {
    display: flex;
    gap: 0.5rem;
  }

  button {
    padding: 0.5rem 1rem;
    font-size: 0.9rem;
    color: var(--fg);
    background: #161b1e;
    border: 1px solid #2a3236;
    border-radius: 6px;
    cursor: pointer;
    transition: background 0.15s, border-color 0.15s;
  }

  button:hover:not(:disabled) {
    background: #1d2428;
    border-color: var(--accent);
  }

  button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
