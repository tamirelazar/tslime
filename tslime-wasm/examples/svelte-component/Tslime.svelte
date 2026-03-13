<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import init, { TslimeWasm } from '../pkg/tslime_wasm.js';

  // Props
  export let width: number = 800;
  export let height: number = 600;
  export let seed: number = 42;
  export let autoStart: boolean = true;
  
  // Internal state
  let canvas: HTMLCanvasElement;
  let tslime: TslimeWasm | null = null;
  let animationFrame: number;
  let isRunning = false;

  onMount(async () => {
    // Initialize WASM module
    await init();
    
    // Set canvas dimensions
    canvas.width = width;
    canvas.height = height;
    
    // Create tslime instance
    tslime = new TslimeWasm(width, height, 'tslime-canvas', seed);
    
    if (autoStart) {
      start();
    }
  });

  onDestroy(() => {
    stop();
    tslime = null;
  });

  function start() {
    if (!tslime || isRunning) return;
    
    isRunning = true;
    tslime.start();
    
    function loop() {
      if (!tslime || !isRunning) return;
      
      tslime.tick();
      animationFrame = requestAnimationFrame(loop);
    }
    
    loop();
  }

  function stop() {
    isRunning = false;
    if (animationFrame) {
      cancelAnimationFrame(animationFrame);
    }
    if (tslime) {
      tslime.stop();
    }
  }

  export function restart() {
    stop();
    if (tslime) {
      tslime = new TslimeWasm(width, height, 'tslime-canvas', Math.floor(Math.random() * 1000000));
    }
    start();
  }
</script>

<div class="tslime-container">
  <canvas
    id="tslime-canvas"
    bind:this={canvas}
    style="width: 100%; height: auto;"
  />
  
  <div class="controls">
    <button on:click={start} disabled={isRunning}>Start</button>
    <button on:click={stop} disabled={!isRunning}>Stop</button>
    <button on:click={restart}>Restart</button>
  </div>
</div>

<style>
  .tslime-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
  }
  
  canvas {
    border: 1px solid #333;
    background: #000;
  }
  
  .controls {
    display: flex;
    gap: 0.5rem;
  }
  
  button {
    padding: 0.5rem 1rem;
    cursor: pointer;
  }
  
  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>
