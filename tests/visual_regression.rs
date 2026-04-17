use std::io::{self, Read};
use std::process::{Command, Stdio};

const GOLDEN_DIR: &str = "tests/golden";

fn get_golden_path(name: &str) -> String {
    format!("{}/{}.txt", GOLDEN_DIR, name)
}

fn capture_print_output(args: &[&str], width: u16, height: u16) -> String {
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--release", "--", "-p"]);
    cmd.args(args);

    let mut child = cmd
        .stdout(Stdio::piped())
        .env("COLUMNS", width.to_string())
        .env("LINES", height.to_string())
        .spawn()
        .expect("Failed to spawn cargo run");

    let mut stdout = child.stdout.take().expect("Failed to get stdout");
    let mut output = String::new();
    stdout
        .read_to_string(&mut output)
        .expect("Failed to read stdout");

    child.wait().expect("Failed to wait for process");

    output
}

fn update_golden(name: &str, content: &str) -> io::Result<()> {
    std::fs::create_dir_all(GOLDEN_DIR)?;
    std::fs::write(get_golden_path(name), content)
}

fn load_golden(name: &str) -> io::Result<String> {
    std::fs::read_to_string(get_golden_path(name))
}

fn normalize_output(output: &str) -> String {
    output
        .lines()
        .filter(|line| !line.starts_with("Compiling"))
        .filter(|line| !line.starts_with("Finished"))
        .filter(|line| !line.contains("warning:"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn should_update_golden() -> bool {
    std::env::var("UPDATE_GOLDEN").is_ok()
}

#[test]
fn test_visual_regression_default_seed() {
    let output = capture_print_output(&["-s", "1"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("default_seed", &normalized).unwrap();
        return;
    }

    match load_golden("default_seed") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("default_seed", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_seed_42() {
    let output = capture_print_output(&["-s", "42"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("seed_42", &normalized).unwrap();
        return;
    }

    match load_golden("seed_42") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("seed_42", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_seed_123() {
    let output = capture_print_output(&["-s", "123"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("seed_123", &normalized).unwrap();
        return;
    }

    match load_golden("seed_123") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("seed_123", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_network_preset() {
    let output = capture_print_output(&["-s", "42", "--preset", "network"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("network_preset", &normalized).unwrap();
        return;
    }

    match load_golden("network_preset") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("network_preset", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_exploratory_preset() {
    let output = capture_print_output(&["-s", "42", "--preset", "exploratory"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("exploratory_preset", &normalized).unwrap();
        return;
    }

    match load_golden("exploratory_preset") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("exploratory_preset", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_heat_palette() {
    let output = capture_print_output(&["-s", "42", "--palette", "heat"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("heat_palette", &normalized).unwrap();
        return;
    }

    match load_golden("heat_palette") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("heat_palette", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_ocean_palette() {
    let output = capture_print_output(&["-s", "42", "--palette", "ocean"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("ocean_palette", &normalized).unwrap();
        return;
    }

    match load_golden("ocean_palette") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("ocean_palette", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_ascii_mode() {
    let output = capture_print_output(&["-s", "42", "--ascii"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("ascii_mode", &normalized).unwrap();
        return;
    }

    match load_golden("ascii_mode") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("ascii_mode", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_braille_mode() {
    let output = capture_print_output(&["-s", "42", "--braille"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("braille_mode", &normalized).unwrap();
        return;
    }

    match load_golden("braille_mode") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: Braille mode output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("braille_mode", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_small_terminal() {
    let output = capture_print_output(&["-s", "42"], 40, 12);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("small_terminal", &normalized).unwrap();
        return;
    }

    match load_golden("small_terminal") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("small_terminal", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_reproducibility_same_seed() {
    let output1 = capture_print_output(&["-s", "999"], 80, 24);
    let output2 = capture_print_output(&["-s", "999"], 80, 24);

    let normalized1 = normalize_output(&output1);
    let normalized2 = normalize_output(&output2);

    assert_eq!(
        normalized1, normalized2,
        "Reproducibility test: same seed should produce identical output"
    );
}

#[test]
fn test_visual_regression_different_seeds_produce_different_output() {
    let output1 = capture_print_output(&["-s", "1000"], 80, 24);
    let output2 = capture_print_output(&["-s", "2000"], 80, 24);

    let normalized1 = normalize_output(&output1);
    let normalized2 = normalize_output(&output2);

    assert_ne!(
        normalized1, normalized2,
        "Different seeds should produce different output"
    );
}

#[test]
fn test_visual_regression_gaussian_diffusion() {
    let output = capture_print_output(
        &[
            "-s",
            "42",
            "--diffusion-kernel",
            "gaussian",
            "--diffusion-sigma",
            "1.0",
        ],
        80,
        24,
    );
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("gaussian_diffusion", &normalized).unwrap();
        return;
    }

    match load_golden("gaussian_diffusion") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: Gaussian diffusion output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("gaussian_diffusion", &normalized).unwrap();
        }
    }
}

#[test]
fn test_fps_invariant_simulation_speed() {
    const REFERENCE_TIME_STEP: f32 = 1.0 / 30.0;
    const STEPS: usize = 100;

    let output_30fps = capture_print_output(&["-s", "42", "--fps", "30", "-n", "500"], 80, 24);
    let output_60fps = capture_print_output(&["-s", "42", "--fps", "60", "-n", "500"], 80, 24);

    let normalized_30 = normalize_output(&output_30fps);
    let normalized_60 = normalize_output(&output_60fps);

    assert_eq!(
        normalized_30, normalized_60,
        "FPS-invariant test failed: --fps 30 and --fps 60 should produce identical output\n\
         With REFERENCE_TIME_STEP={:.4}s and {} simulation steps, both runs should\n\
         produce the same simulation state regardless of target FPS setting.",
        REFERENCE_TIME_STEP, STEPS
    );
}

// ============== ADDITIONAL PRESET TESTS ==============

#[test]
fn test_visual_regression_tendrils_preset() {
    let output = capture_print_output(&["-s", "42", "--preset", "tendrils"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("tendrils_preset", &normalized).unwrap();
        return;
    }

    match load_golden("tendrils_preset") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: tendrils preset output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("tendrils_preset", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_minimal_preset() {
    let output = capture_print_output(&["-s", "42", "--preset", "minimal"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("minimal_preset", &normalized).unwrap();
        return;
    }

    match load_golden("minimal_preset") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: minimal preset output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("minimal_preset", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_moss_preset() {
    let output = capture_print_output(&["-s", "42", "--preset", "moss"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("moss_preset", &normalized).unwrap();
        return;
    }

    match load_golden("moss_preset") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: moss preset output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("moss_preset", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_cosmic_preset() {
    let output = capture_print_output(&["-s", "42", "--preset", "cosmic"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("cosmic_preset", &normalized).unwrap();
        return;
    }

    match load_golden("cosmic_preset") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: cosmic preset output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("cosmic_preset", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_fire_preset() {
    let output = capture_print_output(&["-s", "42", "--preset", "fire"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("fire_preset", &normalized).unwrap();
        return;
    }

    match load_golden("fire_preset") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: fire preset output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("fire_preset", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_zen_preset() {
    let output = capture_print_output(&["-s", "42", "--preset", "zen"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("zen_preset", &normalized).unwrap();
        return;
    }

    match load_golden("zen_preset") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: zen preset output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("zen_preset", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_storm_preset() {
    let output = capture_print_output(&["-s", "42", "--preset", "storm"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("storm_preset", &normalized).unwrap();
        return;
    }

    match load_golden("storm_preset") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: storm preset output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("storm_preset", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_river_preset() {
    let output = capture_print_output(&["-s", "42", "--preset", "river"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("river_preset", &normalized).unwrap();
        return;
    }

    match load_golden("river_preset") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: river preset output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("river_preset", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_ethereal_preset() {
    let output = capture_print_output(&["-s", "42", "--preset", "ethereal"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("ethereal_preset", &normalized).unwrap();
        return;
    }

    match load_golden("ethereal_preset") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: ethereal preset output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("ethereal_preset", &normalized).unwrap();
        }
    }
}

// ============== ADDITIONAL PALETTE TESTS ==============

#[test]
fn test_visual_regression_forest_palette() {
    let output = capture_print_output(&["-s", "42", "--palette", "forest"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("forest_palette", &normalized).unwrap();
        return;
    }

    match load_golden("forest_palette") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: forest palette output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("forest_palette", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_neon_palette() {
    let output = capture_print_output(&["-s", "42", "--palette", "neon"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("neon_palette", &normalized).unwrap();
        return;
    }

    match load_golden("neon_palette") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: neon palette output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("neon_palette", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_warm_palette() {
    let output = capture_print_output(&["-s", "42", "--palette", "warm"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("warm_palette", &normalized).unwrap();
        return;
    }

    match load_golden("warm_palette") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: warm palette output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("warm_palette", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_vibrant_palette() {
    let output = capture_print_output(&["-s", "42", "--palette", "vibrant"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("vibrant_palette", &normalized).unwrap();
        return;
    }

    match load_golden("vibrant_palette") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: vibrant palette output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("vibrant_palette", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_slime_palette() {
    let output = capture_print_output(&["-s", "42", "--palette", "slime"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("slime_palette", &normalized).unwrap();
        return;
    }

    match load_golden("slime_palette") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: slime palette output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("slime_palette", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_mold_palette() {
    let output = capture_print_output(&["-s", "42", "--palette", "mold"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("mold_palette", &normalized).unwrap();
        return;
    }

    match load_golden("mold_palette") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: mold palette output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("mold_palette", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_fungus_palette() {
    let output = capture_print_output(&["-s", "42", "--palette", "fungus"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("fungus_palette", &normalized).unwrap();
        return;
    }

    match load_golden("fungus_palette") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: fungus palette output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("fungus_palette", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_swamp_palette() {
    let output = capture_print_output(&["-s", "42", "--palette", "swamp"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("swamp_palette", &normalized).unwrap();
        return;
    }

    match load_golden("swamp_palette") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: swamp palette output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("swamp_palette", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_moss_palette() {
    let output = capture_print_output(&["-s", "42", "--palette", "moss"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("moss_palette", &normalized).unwrap();
        return;
    }

    match load_golden("moss_palette") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: moss palette output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("moss_palette", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_cosmic_palette() {
    let output = capture_print_output(&["-s", "42", "--palette", "cosmic"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("cosmic_palette", &normalized).unwrap();
        return;
    }

    match load_golden("cosmic_palette") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: cosmic palette output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("cosmic_palette", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_ethereal_palette() {
    let output = capture_print_output(&["-s", "42", "--palette", "ethereal"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("ethereal_palette", &normalized).unwrap();
        return;
    }

    match load_golden("ethereal_palette") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: ethereal palette output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("ethereal_palette", &normalized).unwrap();
        }
    }
}

// ============== MODE AND FEATURE TESTS ==============

#[test]
fn test_visual_regression_halfblock_mode() {
    let output = capture_print_output(&["-s", "42", "--halfblock"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("halfblock_mode", &normalized).unwrap();
        return;
    }

    match load_golden("halfblock_mode") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: halfblock mode output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("halfblock_mode", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_quadrant_mode() {
    let output = capture_print_output(&["-s", "42", "--quadrant"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("quadrant_mode", &normalized).unwrap();
        return;
    }

    match load_golden("quadrant_mode") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: quadrant mode output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("quadrant_mode", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_truecolor_mode() {
    let output = capture_print_output(&["-s", "42", "--colors", "true"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("truecolor_mode", &normalized).unwrap();
        return;
    }

    match load_golden("truecolor_mode") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: truecolor mode output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("truecolor_mode", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_dither_mode() {
    let output = capture_print_output(&["-s", "42", "--dither"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("dither_mode", &normalized).unwrap();
        return;
    }

    match load_golden("dither_mode") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: dither mode output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("dither_mode", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_motion_blur() {
    let output = capture_print_output(&["-s", "42", "--trail-history", "5"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("motion_blur", &normalized).unwrap();
        return;
    }

    match load_golden("motion_blur") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: motion blur output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("motion_blur", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_auto_normalize() {
    let output = capture_print_output(&["-s", "42", "--auto-normalize"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("auto_normalize", &normalized).unwrap();
        return;
    }

    match load_golden("auto_normalize") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: auto normalize output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("auto_normalize", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_invert_palette() {
    let output = capture_print_output(&["-s", "42", "--invert-palette"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("invert_palette", &normalized).unwrap();
        return;
    }

    match load_golden("invert_palette") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: invert palette output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("invert_palette", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_palette_shift() {
    let output = capture_print_output(&["-s", "42", "--palette-shift", "15"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("palette_shift", &normalized).unwrap();
        return;
    }

    match load_golden("palette_shift") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: palette shift output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("palette_shift", &normalized).unwrap();
        }
    }
}

// ============== WINDOW MODE TESTS ==============

#[test]
fn test_visual_regression_window_minimal_frame() {
    // Default windowed mode with minimal chrome (the new default as of window-mode).
    // Print mode renders edge-to-edge regardless, so this verifies the flag is
    // accepted without error and produces stable output.
    let output = capture_print_output(&["-s", "42", "--chrome-style", "minimal"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("window_minimal_frame", &normalized).unwrap();
        return;
    }

    match load_golden("window_minimal_frame") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: window minimal frame output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("window_minimal_frame", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_window_fullscreen() {
    // Fullscreen mode (--fullscreen) opts out of the window frame, rendering
    // edge-to-edge (same as pre-v0.2 behaviour). Verifies the flag is accepted
    // without error and produces stable output.
    let output = capture_print_output(&["-s", "42", "--fullscreen"], 80, 24);
    let normalized = normalize_output(&output);

    if should_update_golden() {
        update_golden("window_fullscreen", &normalized).unwrap();
        return;
    }

    match load_golden("window_fullscreen") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: window fullscreen output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!(
                "Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept."
            );
            update_golden("window_fullscreen", &normalized).unwrap();
        }
    }
}
