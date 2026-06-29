<script lang="ts">
  import Tslime from './lib/Tslime.svelte';
</script>

<main>
  <header>
    <h1>tslime</h1>
    <p>
      A lightweight terminal screensaver that simulates the growth of
      <em>Physarum polycephalum</em> — slime mold. This page runs the very same
      simulation in your browser, compiled to WebAssembly and drawn with WebGL2.
    </p>
  </header>

  <Tslime width={800} height={600} />

  <footer>
    <a href="https://github.com/tamirelazar/tslime">github.com/tamirelazar/tslime</a>
  </footer>
</main>
