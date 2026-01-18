# Changelog

All notable changes to tslime are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Comprehensive README with installation and usage documentation
- Man page documentation (docs/tslime.1.md)
- Contributing guide (CONTRIBUTING.md)
- Screenshots directory with example outputs
- Demo tape file for GIF generation
- 8 new presets: minimal, moss, cosmic, fire, zen, storm, river, ethereal
- 4 new color palettes: legiblemono, moss, cosmic, ethereal
- Custom palette support via hex color strings
- WebM video export with FFmpeg
- Mouse interaction modes (attract/repel)
- Terrain effects (smooth, turbulent, mixed)
- Wind simulation
- Species mode with multiple agent types
- Obstacle support (circle, rect, image-based)
- Motion blur / trail history
- Adaptive brightness normalization
- Dithering modes (ordered, error diffusion)
- Grid overlay options
- Runtime parameter adjustment via keyboard
- Undo/redo for parameter changes
- Config save/load functionality
- Comprehensive parameter validation with bounds checking

### Changed
- Updated noise crate from 0.2 to 0.9 (fixes future Rust compatibility)
- Improved error handling throughout codebase
- Enhanced CLI validation with helpful error messages

### Fixed
- Clippy warnings (manual_range_contains, manual_find)
- Duplicate build targets for memory benchmarks
- Unsafe unwrap() calls in WebM export and frame capture
- Future-incompatibility warnings from deprecated num-* crates
- Duplicate code blocks in `trail_map.rs` and `timing.rs`

### Documentation
- Completed API documentation for all public modules (100% coverage)
- Updated README with all 16 palettes and 12 presets
- Added module-level documentation
- Fixed intra-doc links in library documentation

## [0.1.0] - 2024-12-24

### Added
- Initial release
- Physarum simulation based on Jeff Jones particle model
- Four presets: network, exploratory, tendrils, organic
- Four color palettes: organic, heat, ocean, mono
- Multiple character modes: half-block, ASCII, braille
- Screensaver, live, and print modes
- Configurable simulation parameters
- 256-color ANSI rendering with half-block characters
- Visual regression tests
- CI/CD pipeline with GitHub Actions
- Cross-platform support (Linux, macOS, Windows)

### Technical
- Rust implementation with zero runtime dependencies
- crossterm for cross-platform terminal handling
- clap for CLI argument parsing
- rand_xoshiro for fast, seedable PRNG
- Optimized for low CPU and memory usage
