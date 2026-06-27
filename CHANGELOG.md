# Changelog

All notable changes to this project will be documented in this file.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2026-06-27

Windows distribution fix.

### Fixed
- Windows release binary is now statically linked against the MSVC C runtime
  (`+crt-static`). The previous build dynamically linked `vcruntime140.dll`, so
  `tslime.exe` failed to launch on a clean Windows without the VC++ 2015–2022
  redistributable — silently, with no error, making `tslime` appear to do
  nothing. This restores the single-static-binary guarantee on Windows.
- README Windows install snippet now handles an unset user `PATH` (avoids
  writing a leading `;`).

## [0.1.1] - 2026-06-26

Distribution and release-pipeline fixes.

### Changed
- Linux release binary is now statically linked (musl target), so the single
  `tslime-linux-x86_64` download runs on any x86_64 Linux distribution
  regardless of glibc version.
- Install documentation now leads with `cargo install` and a Homebrew tap;
  see the README for per-platform paths.

### Fixed
- macOS distribution: prebuilt macOS binaries are no longer published (the
  previous one was Gatekeeper-blocked and mislabeled as x86_64 while actually
  being arm64). Install on macOS via `brew install tamirelazar/tslime/tslime`
  or `cargo install tslime`, both of which avoid Gatekeeper.

### Added
- Homebrew tap `tamirelazar/homebrew-tslime` (source build).
- One-paste install blocks for Linux and Windows in the README: they fetch the
  latest release binary, install it to a user directory, put it on `PATH`, and
  (on Windows) clear the Mark-of-the-Web so SmartScreen stays quiet.
- Nix flake: `nix run github:tamirelazar/tslime` runs tslime ephemerally without
  installing; `nix build` and `nix profile install` are also supported.
- Docker image on GHCR: `docker run --rm -it ghcr.io/tamirelazar/tslime` runs
  tslime ephemerally without installing (published on release; linux/amd64).

## [0.1.0] - 2026-06-26

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
