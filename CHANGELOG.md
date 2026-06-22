# Changelog

All notable changes to this project will be documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - TBD

First public release.

### Added
- Physarum simulation (Jones 2010 model) with 30 presets and runtime parameter controls
- Terminal rendering: half-block, ASCII, braille, quadrant, shade, points, and sculpted charsets; 22 OKLch-based color palettes
- Screensaver and interactive modes; pause, restart, preset/palette cycling at runtime
- GIF, PNG, and WebM export
- Window-frame display modes
- `vinescii` preset — the vines (flocking) pattern in pure ASCII
- Experimental: multi-species, choir audio, and GUI (feature-gated); WASM build (standalone crate)

### Changed
- Preset pass: renamed pulse→slime, flocking→vines, ripple→smoke, lumen→mold
  (old names still accepted as CLI aliases); per-preset visual tuning applied
  (braille/quadrant charsets, palette assignments, auto-normalize, window frames);
  constellation re-rolls its init layout on reset and auto-resets on collapse.
- Fixed the stale `--preset` help text (was listing removed presets).
