# tslime

A terminal screensaver that runs a Physarum polycephalum (slime mold) transport-network
simulation. Tens of thousands of agents deposit and follow pheromone trails, and the
resulting network is drawn straight into your terminal. It ships as a single static
binary, runs on Linux, macOS, Windows, and over SSH, and works as both a screensaver
and an instrument: every simulation parameter can be steered live from the keyboard
while it runs.

<!-- ws6: hero gif -->

## Install

From crates.io:

```bash
cargo install tslime
```

From source (requires Rust 1.70 or later):

```bash
git clone https://github.com/tamirelazar/tslime.git
cd tslime
cargo build --release
./target/release/tslime
```

Prebuilt binaries for Linux, macOS, and Windows accompany each GitHub release.

## Usage

Run with no arguments for the interactive mode:

```bash
tslime
```

Screensaver mode exits on the first keypress:

```bash
tslime -S
```

Some starting points:

```bash
tslime --preset lightning --palette heat      # fast dendritic branching, warm colors
tslime --preset zen --palette ocean --fps 24  # slow and quiet
tslime --seed 42 --population 80000           # reproducible run, denser network
tslime --braille --palette mono               # high-resolution monochrome
tslime --palette-cycles 3 --palette-cycle-mode mirror  # banded contour coloring
tslime --ascii --glyph-selection hybrid                # edge-aware directional glyphs on filaments
tslime --export-gif demo.gif --export-frames 120 --export-fps 30
tslime -p > frame.txt                         # print a single frame and exit
```

`tslime --help` lists every flag. `tslime --explain` walks through what each
simulation parameter does and how the parameters interact.

## Controls

Everything below works while the simulation is running. For paired keys,
lowercase increases and uppercase (Shift) decreases.

### Simulation

| Key | Action |
|---|---|
| `Space` | Pause / resume |
| `r` | Restart |
| `1`–`7` | Select preset (`Shift+1`–`7` to compare against current) |
| `8` | Randomize parameters |
| `0` | Reset to defaults |
| `+` / `-` | Time scale |
| `a` / `A` | Sensor angle |
| `j` / `J` | Sensor distance |
| `t` / `T` | Turn angle |
| `s` / `S` | Step size |
| `e` / `E` | Trail decay |
| `i` / `I` | Deposit amount |
| `k` | Cycle diffusion kernel |
| `w` / `W` | Cycle wind direction |
| `u` | Cycle terrain type |
| `y` / `Y` | Terrain strength |

### Appearance

| Key | Action |
|---|---|
| `c` / `C` | Cycle palette |
| `9` / `*` | Cycle theme |
| `` ` `` / `~` | Cycle character set |
| `"` | Cycle color anti-aliasing (active charset) |
| `n` / `N` | Brightness |
| `m` / `M` | Cycle intensity mapping |
| `x` | Invert palette |
| `z` | Reverse palette |
| `o` | Palette hue-shift speed |
| `v` | Motion blur |
| `'` | Trail age coloring |
| `.` | Growth/decay highlighting |
| `>` | Edge glow |
| `(` / `)` | Cycle window frame |
| `F10` | Cycle chrome (minimal / expanded / fullscreen) |
| `F11` | Fullscreen |

### System

| Key | Action |
|---|---|
| `p` | Palette editor |
| `Ctrl+S` / `Ctrl+L` | Save / load configuration |
| `Ctrl+Z` / `Ctrl+Y` | Undo / redo |
| `h` | Controls panel |
| `?` | Keyboard hints |
| `\` | Dashboard overlay |
| `g` | Save current frame to PNG |
| `q` | Quit |
| `Esc` | Close open overlay |

## Gallery

<!-- ws6: preset/palette showcase gifs -->

There are 34 named presets: network, exploratory, tendrils, organic, minimal, moss,
cosmic, fire, zen, storm, river, ethereal, petri, vortex, lightning, crystal,
chaosedge, blob, worm, pulse, coral, flocking, maze, ripple, vortex36, chameleon,
dynamictendrils, morphingcoral, reactiveswarm, duelingmodulators, lumen, aurora,
bloom, and etching. Palettes and character sets are independent of the preset and
can be cycled at runtime.

Four showcase presets highlight specific visual levers:

- **lumen** — Bleuje-style front-lit veins: temporal-accent recolor of growing fronts.
- **aurora** — Luminous network glow: afterglow + soft diffusion + long faint tails.
- **bloom** — Banded coral depth via mirrored palette cycles.
- **etching** — Directional filament linework via Sobel glyph selection (Braille, TUI-only).

## How it works

The simulation implements the agent-based model from Jones (2010) [1]. Each agent
carries only a position and a heading. Every step it senses the pheromone trail at
three points ahead of it (front-left, front, front-right), rotates toward the
strongest reading, moves forward, and deposits pheromone at its new position.

The trail lives in a 2D grid. Each frame the grid is diffused with a 3×3 mean
kernel (a 5×5 Gaussian is available) and then decayed by a constant factor. Agents
reinforce paths that other agents have taken; diffusion and decay erase paths that
go unused. Transport networks emerge from nothing but these local rules.

To draw a frame, the grid is downsampled to the terminal's cell dimensions by
average pooling, then mapped to characters: half-blocks by default for double
vertical resolution, with ASCII, braille, quadrant, shade, and other character sets
available.

Colors come from palettes defined as gradients in the OKLch color space [5], which
keeps perceived brightness uniform across the gradient — intensity in the
simulation reads as intensity on screen, regardless of hue.

## Experimental — help wanted

Features that exist but aren't ready for the default experience. Each has a
tracking issue describing current state and what's needed — contributions welcome.

| Feature | Try it | Issue |
|---|---|---|
| Multi-species simulation | `cargo install tslime --features multi-species`, then `--species 'red:20k:ff0000' --species 'blue:20k:0000ff' --species-colors` | #8 |
| Choir mode (audio) | `cargo install tslime --features audio`, then `--choir` | #9 |
| GUI mode | `cargo build --features gui` | #10 |
| WASM build | `tslime-wasm/` (standalone crate) | #11 |
| Dithering | hidden flags: `--dither-mode ordered` (and `d`/`D`/`[`/`]` keys once enabled) | #12 |

## Contributing

Bug reports, terminal compatibility notes, and work on the experimental features
are all welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for setup, testing, and
the lay of the codebase, or pick up one of the open
[issues](https://github.com/tamirelazar/tslime/issues).

## License

MIT. See [LICENSE](LICENSE).

## References

1. Jones, J. (2010). "Characteristics of Pattern Formation and Evolution in Approximations of Physarum Transport Networks." *Artificial Life*, 16(2), 127-153. doi:10.1162/artl.2010.16.2.16202
2. Miranda, E. R., Adamatzky, A., & Jones, J. (2011). "Sounds Synthesis with Slime Mould of Physarum Polycephalum." *Journal of Bionic Engineering*, 8(2), 107-113. doi:10.1016/S1672-6529(11)60016-4
3. Floyd, R. W., & Steinberg, L. (1976). "An Adaptive Algorithm for Spatial Grayscale." *Proceedings of the Society for Information Display*, 17(2), 75-77.
4. Bayer, B. E. (1973). "An optimum method for two-level rendition of continuous-tone pictures." *IEEE International Conference on Communications*, Vol. 1, 11-15.
5. Ottosson, B. (2020). "A perceptual color space for image processing." https://bottosson.github.io/posts/oklab/

Thanks to [cbonsai](https://gitlab.com/jallbrit/cbonsai) for the terminal-art
inspiration and [crossterm](https://github.com/crossterm-rs/crossterm) for
terminal handling.
