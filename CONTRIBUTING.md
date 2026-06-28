# Contributing to tslime

Contributions are welcome — bug fixes, new presets and palettes, terminal
compatibility reports, performance work, and progress on the experimental
features. If you're unsure whether something fits, open an issue first and ask.

## Where help is wanted

- **Experimental features.** Multi-species simulation (#8), choir audio (#9),
  GUI mode (#10), WASM build (#11), and dithering (#12) all exist behind gates
  and need work before they join the default experience. Each tracking issue
  describes the current state.
- **Preset and palette feedback.** If a preset collapses, stalls, or just looks
  wrong in your terminal, an issue with the preset name, seed, and terminal
  details is genuinely useful.
- **Terminal compatibility reports.** tslime targets Linux, macOS, Windows, and
  SSH sessions. Reports from unusual terminals, multiplexers, and font setups
  help.

## Development setup

Requires Rust 1.70 or later.

```bash
git clone https://github.com/tamirelazar/tslime.git
cd tslime
cargo build
cargo run -- -S
```

## Testing

```bash
cargo test --lib                          # unit tests
cargo test --test visual_regression      # golden-file tests (tests/golden/)
cargo bench                               # criterion benchmarks
```

If a change intentionally alters visual output, regenerate the golden files:

```bash
UPDATE_GOLDEN=true cargo test --test visual_regression
```

Commit golden changes only when the output change is intentional — review the
diff and make sure it shows what you meant to change.

## Feature flags

The default build enables only the `terminal` feature. Experimental features
are opt-in:

| Feature | What it adds | Issue | Build |
|---|---|---|---|
| `multi-species` | Multiple agent species with independent parameters | #8 | `cargo build --features multi-species` |
| `audio` | Choir mode: sonifies the simulation via cpal | #9 | `cargo build --features audio` |
| `gui` | Windowed GUI mode built on iced | #10 | `cargo build --features gui` |

## Architecture

```
src/
├── cli.rs          # clap argument definitions, validation, mode dispatch
├── app/            # run loop, mode handlers, --explain output
├── simulation/     # agents (sense/rotate/move/deposit), trail map, presets, config
├── render/         # downsampling, character sets, OKLch palettes, overlays/panels
├── terminal/       # screen lifecycle, input polling, frame buffer, runtime controls
└── export/         # GIF, PNG, and WebM export
```

Data flow per frame: agents sense the trail map, rotate, move, and deposit
pheromone; the trail map is diffused and decayed; the grid is downsampled to
terminal dimensions; cells are mapped through the active character set and
palette; the result is written to stdout as a single ANSI frame.

## Before submitting

```bash
cargo fmt
cargo clippy -- -D warnings
cargo test
```

Work on a feature branch (`feat/...`, `fix/...`, `chore/...`) and open a PR
against `dev`. Commit messages use the form `type: description`, where type
is one of `feat`, `fix`, `refactor`, `perf`, `test`, `docs`, or `chore`.
