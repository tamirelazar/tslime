# Changelog

All notable changes to tslime are documented in this file.

## [Unreleased]

### Added
- Comprehensive README with installation and usage documentation
- Man page documentation (docs/tslime.1.md)
- Contributing guide (CONTRIBUTING.md)
- Screenshots directory with example outputs
- Demo tape file for GIF generation

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
