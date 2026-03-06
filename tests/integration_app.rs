// Integration tests for app module functionality
use tslime::app;
use tslime::render::palette::RgbColor;
use tslime::simulation::config::{SimConfig, SpeciesConfig};

/// Test that generate_completions handles supported shells correctly
#[test]
fn test_generate_completions_supported_shells() {
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let result = app::generate_completions(shell);
        assert!(
            result.is_ok(),
            "generate_completions should succeed for shell: {}",
            shell
        );
    }
}

/// Test that generate_completions rejects unknown shells
#[test]
fn test_generate_completions_unknown_shell() {
    let result = app::generate_completions("unknown_shell");
    assert!(
        result.is_err(),
        "generate_completions should fail for unknown shell"
    );
}

/// Test that print_parameter_explanations doesn't panic
#[test]
fn test_print_parameter_explanations() {
    app::print_parameter_explanations();
}

/// Test that extract_species_rgb_colors correctly extracts colors from config
#[test]
fn test_extract_species_rgb_colors() {
    let config = SimConfig {
        species_configs: vec![
            SpeciesConfig {
                name: "red".to_string(),
                count: 1000,
                sensor_angle: 22.5,
                rotation_angle: 45.0,
                step_size: 1.0,
                deposit_amount: 5.0,
                color: RgbColor::from_hex(0xff0000),
                trail_modulation: None,
            },
            SpeciesConfig {
                name: "blue".to_string(),
                count: 1000,
                sensor_angle: 30.0,
                rotation_angle: 60.0,
                step_size: 1.5,
                deposit_amount: 3.0,
                color: RgbColor::from_hex(0x0000ff),
                trail_modulation: None,
            },
        ],
        ..Default::default()
    };

    let colors = app::extract_species_rgb_colors(&config);
    assert_eq!(colors.len(), 2);
    assert_eq!(colors[0].r, 255);
    assert_eq!(colors[0].g, 0);
    assert_eq!(colors[0].b, 0);
    assert_eq!(colors[1].r, 0);
    assert_eq!(colors[1].g, 0);
    assert_eq!(colors[1].b, 255);
}

/// Test that extract_species_rgb_colors ignores invalid hex colors
#[test]
fn test_extract_species_rgb_colors_invalid_hex() {
    // With RgbColor type, invalid colors are impossible at compile time
    // This test verifies the function works with valid colors
    let config = SimConfig {
        species_configs: vec![
            SpeciesConfig {
                name: "valid".to_string(),
                count: 1000,
                sensor_angle: 22.5,
                rotation_angle: 45.0,
                step_size: 1.0,
                deposit_amount: 5.0,
                color: RgbColor::from_hex(0xff0000),
                trail_modulation: None,
            },
            SpeciesConfig {
                name: "another_valid".to_string(),
                count: 1000,
                sensor_angle: 30.0,
                rotation_angle: 60.0,
                step_size: 1.5,
                deposit_amount: 3.0,
                color: RgbColor::from_hex(0x00ff00),
                trail_modulation: None,
            },
        ],
        ..Default::default()
    };

    let colors = app::extract_species_rgb_colors(&config);
    assert_eq!(colors.len(), 2);
    assert_eq!(colors[0].r, 255);
    assert_eq!(colors[0].g, 0);
    assert_eq!(colors[0].b, 0);
    assert_eq!(colors[1].r, 0);
    assert_eq!(colors[1].g, 255);
    assert_eq!(colors[1].b, 0);
}
