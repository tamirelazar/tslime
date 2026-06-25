# tslime

A terminal screensaver that runs a Physarum polycephalum (slime mold) transport-network
simulation. Tens of thousands of agents deposit and follow pheromone trails, and the
resulting network is drawn straight into your terminal. It ships as a single static
binary, runs on Linux, macOS, Windows, and over SSH, and works as both a screensaver
and an instrument: every simulation parameter can be steered live from the keyboard
while it runs.

<p align="center">
  <img src="assets/demos/hero.gif" alt="tslime growing the logo across the Organic, Constellation, Vinescii, and Trademark presets" width="100%">
</p>

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
tslime --preset drift --palette ocean --fps 24  # slow and quiet
tslime --seed 42 --population 80000           # reproducible run, denser network
tslime --braille --palette mono               # high-resolution monochrome
tslime --palette-cycles 3 --palette-cycle-mode mirror  # banded contour coloring
tslime --ascii --glyph-selection hybrid                # edge-aware directional glyphs on filaments
tslime --transition figlet                             # announce preset switches with a big block-letter name
tslime --transition type --transition-tagline          # typed readout + the preset's one-line tagline
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
| `1`–`4` | Switch preset (Organic / Constellation / Vinescii / Trademark) |
| `5`–`7` | Custom binds — set in `~/.config/tslime/keybinds.toml` |
| `Shift+1`–`7` | Compare bound preset/config (A/B) |
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

### Controls panel navigation

The Controls panel opens into the Console study view (full master-detail interface) and supports two-depth navigation:

| Key | Action |
|---|---|
| `Tab` | Toggle between Tuner (bottom strip) and Console (full panel) |
| `← / →` | Adjust the focused parameter (equivalent to its per-parameter hotkey) |
| `↑ / ↓` | Move focus between parameters |
| `↵` | Activate an action parameter (Reset / Save Frame / Randomize) |
| `[ / ]` | Move to previous / next category |
| `{ / }` | Adjust dither intensity (development feature) |
| `Esc` | Close the panel |

Note: The per-parameter hotkeys (`a`/`A`, `j`/`J`, `t`/`T`, etc. from the Simulation and Appearance tables above) continue to work anywhere and take precedence. Arrow keys provide keyboard-driven alternatives to the focused-param hotkey while the panel is open.

### Custom Keybinds

Customize quick-keys `1`–`7` by creating a `~/.config/tslime/keybinds.toml` file with the following format:

```toml
[[keybind]]
key = "4"
preset = "fire"

[[keybind]]
key = "5"
config = "my-night-config"
```

- **Keys**: Bind to any digit `1`–`7`. Keys `1`–`4` default to Organic, Constellation, Vinescii, and Trademark; user entries override.
- **Targets**: Bind to either a `preset` (any of the 31 named presets) or a `config` (any saved configuration from `Ctrl+S`).
- **Comparison**: Press `Shift+1` through `Shift+7` to compare the bound preset or config against the current settings (A/B mode).
- **Invalid entries**: Silently ignored; the app launches normally and skips unparseable lines.
- **Live bindings**: The `?` overlay shows current key bindings and their targets.

## Gallery

<!-- ws6: preset/palette showcase gifs -->

There are 31 named presets: network, exploratory, tendrils, organic, fire, river,
petri, vortex, lightning, chaosedge, blob, slime, vines, vinescii, smoke, vortex36,
dynamictendrils, mold, etching, drift, constellation, mosaic, marble, prism, vellum,
forge, wane, gossamer, codex, tide, and trademark. The old names pulse, flocking, ripple, and
lumen are still accepted as CLI aliases. Palettes and character sets are independent of
the preset and can be cycled at runtime. Preset parameters are defined in
`src/simulation/config.rs` and `src/preset_sim_defaults.rs`.

Showcase presets highlight specific visual levers:

- **slime** — Pulsing network with strong deposits and the slime palette; auto-normalizes brightness.
- **vines** — Flocking-pattern filaments rendered without a window frame.
- **vinescii** — The vines pattern rendered in pure ASCII.
- **smoke** — Diffuse drift with the slate palette; auto-normalizes brightness.
- **mold** — Bleuje-style front-lit veins with the mold palette; auto-normalizes brightness.
- **etching** — Directional filament linework via Sobel glyph selection (Braille, TUI-only).
- **drift** — Color that shifts with motion direction (temporal Hue mode).
- **constellation** — Sparse star-map scatter via the Points charset; re-rolls layout on reset.
- **trademark** (alias `logo`) — The tslime logo held as a stable figure: constellation re-stamp behavior with the embedded logo image as the template (Ethereal palette, HalfBlock, auto-normalized brightness). Bound to key `4` by default.
- **mosaic** — Posterized color bands: Quantize mapping + wrapped palette cycles.
- **marble** — Veined stone via heavy Gaussian diffusion + Perlin intensity mapping.
- **prism** — Maximum color resolution: HalfBlockDual charset + strong color anti-aliasing.
- **vellum** — Soft parchment density: Shade charset + logarithmic deposit curve.
- **forge** — Grainy molten thermal: exponential intensity mapping + afterglow.
- **wane** — Slow ghosting decay via low decay-gamma + power deposit curve.
- **gossamer** — Delicate Braille threads with brightness glyphs + power mapping.
- **codex** — Typographic engraving: custom ASCII charset + sigmoid contrast.
- **tide** — Living water with animated hue-shift over time.

Six new palettes accompany them: **jade** (mid-saturation green), **amber** (warm
earth), **slate** (cool stone grey), **pastel** (high-key airy), **ink** (duotone
ink-on-paper), and **copper** (oxidized rust-to-verdigris).

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
| Dithering | hidden flags: `--dither-mode ordered` (and `d`/`D`/`{`/`}` keys once enabled) | #12 |

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
