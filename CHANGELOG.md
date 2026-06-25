# Changelog

All notable changes to this project will be documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - TBD

First public release.

### Added
- Physarum simulation (Jones 2010 model) with 31 presets and runtime parameter controls
- `trademark` preset (alias `logo`) — the tslime logo held as a stable figure (constellation re-stamp behavior with the embedded logo image as the template); bound to quick-key `4` by default
- Terminal rendering: half-block, ASCII, braille, quadrant, shade, points, and sculpted charsets; 22 OKLch-based color palettes
- Screensaver and interactive modes; pause, restart, preset/palette cycling at runtime
- GIF, PNG, and WebM export
- Window-frame display modes
- README hero demo: a looping montage that grows the tslime logo across the
  Organic, Constellation, Vinescii, and Trademark launch presets
- README gallery: six looping demos of the runtime controls — palette cycling,
  character sets, preset transitions, randomize, live parameter tuning, and the
  palette editor (replaces the earlier static preset stills)
- No-notifications mode: suppress transient toasts and ambient parameter
  readouts for a clean field. Start with `--no-notifications`; toggle live with
  `Ctrl+N`. User-opened overlays (controls, palette editor, dashboard, preset
  transitions) are unaffected.
- `vinescii` preset — the vines (flocking) pattern in pure ASCII
- Experimental: multi-species, choir audio, and GUI (feature-gated); WASM build (standalone crate)

### Changed
- Quick-keys `1`–`4` now switch the launch presets (Organic, Constellation, Vinescii, Trademark).
- Preset pass: renamed pulse→slime, flocking→vines, ripple→smoke, lumen→mold
  (old names still accepted as CLI aliases); per-preset visual tuning applied
  (braille/quadrant charsets, palette assignments, auto-normalize, window frames);
  constellation re-rolls its init layout on reset and auto-resets on collapse.
- Fixed the stale `--preset` help text (was listing removed presets).

### Added (Custom Keybinds)
- Custom key bindings via `~/.config/tslime/keybinds.toml`: bind keys `1`–`7` to any preset or
  saved config; user binds override the defaults. A/B compare (`Shift+1`–`7`) works for bound
  presets and configs. The `?` overlay shows live bindings.
