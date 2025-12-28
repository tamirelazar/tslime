# Contributing to tslime

Thank you for your interest in contributing to tslime!

## Development Setup

### Prerequisites

- Rust 1.70 or later
- Git

### Getting Started

```bash
# Fork the repository
git clone https://github.com/yourusername/tslime.git
cd tslime

# Install development dependencies
cargo install cargo-edit
cargo install cargo-watch  # Optional, for live development
```

### Development Workflow

```bash
# Watch for changes and run tests
cargo watch -x test -x clippy

# Build and run in debug mode
cargo run -- -S

# Build release binary
cargo build --release
```

## Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_agent_rotate

# Run visual regression tests
cargo test --test visual_regression
```

## Code Style

### Formatting

```bash
# Check formatting
cargo fmt --check

# Format code
cargo fmt
```

### Linting

```bash
# Run clippy (warnings as errors, required by CI)
cargo clippy -- -D warnings

# Run clippy without treating warnings as errors
cargo clippy
```

## Project Structure

```
src/
├── main.rs              # Entry point, CLI handling
├── simulation/
│   ├── mod.rs           # Simulation orchestration
│   ├── agent.rs         # Agent struct and behavior
│   ├── trail_map.rs     # 2D pheromone grid + diffusion
│   └── config.rs        # Simulation parameters
├── render/
│   ├── mod.rs           # Rendering orchestration
│   ├── downsample.rs    # Grid → terminal cell mapping
│   ├── charset.rs       # Character selection logic
│   └── palette.rs       # Color palette definitions
├── terminal/
│   ├── mod.rs           # Terminal setup/teardown
│   ├── screen.rs        # Alternate buffer management
│   ├── input.rs         # Non-blocking input polling
│   └── output.rs       # Terminal output
└── cli.rs              # CLI argument parsing
```

## Adding Features

### Adding a New Preset

1. Add preset variant to `simulation/config.rs`:

```rust
pub enum Preset {
    Network,
    Exploratory,
    Tendrils,
    Organic,
    YourNewPreset,  // Add here
}
```

2. Add configuration in `From<Preset>` implementation:

```rust
Preset::YourNewPreset => Self {
    population: 40_000,
    sensor_angle: 30.0,
    sensor_distance: 12.0,
    rotation_angle: 45.0,
    step_size: 1.0,
    decay_factor: 0.90,
    deposit_amount: 4.0,
    diffusion_kernel: DiffusionKernel::Mean3x3,
},
```

3. Add string parsing in `cli.rs`:

```rust
"yourpreset" => Ok(Preset::YourNewPreset),
```

### Adding a New Color Palette

1. Add palette variant to `render/palette.rs`:

```rust
pub enum Palette {
    Organic,
    Heat,
    Ocean,
    Mono,
    YourNewPalette,  // Add here
}
```

2. Implement color gradient logic in palette module.

3. Update `cli.rs` parsing and documentation.

## Testing

### Unit Tests

Add unit tests in the same file using `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_function() {
        let result = your_function();
        assert_eq!(result, expected);
    }
}
```

### Visual Regression Tests

Update golden tests in `tests/golden/`:

```bash
# Update golden files (when making intentional changes)
UPDATE_GOLDEN=true cargo test --test visual_regression

# Run tests (verify against golden files)
cargo test --test visual_regression
```

## Performance Guidelines

- **No allocations in hot loops**: Pre-allocate all buffers
- **Cache-friendly access**: Use row-major iteration for 2D arrays
- **Batch operations**: Minimize function calls in tight loops
- **Profile first**: Use `cargo flamegraph` before optimizing

## Submitting Changes

1. Create a branch from `main`
2. Make your changes with clear commit messages
3. Ensure all tests pass (`cargo test`)
4. Ensure clippy passes (`cargo clippy -- -D warnings`)
5. Ensure formatting is correct (`cargo fmt --check`)
6. Push to your fork and create a Pull Request

## Reporting Issues

When reporting issues, please include:

- OS and terminal emulator
- Version (`tslime --version`)
- Steps to reproduce
- Expected vs actual behavior
- Screenshot if applicable

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
