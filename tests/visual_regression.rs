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
    stdout.read_to_string(&mut output).expect("Failed to read stdout");
    
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

#[test]
fn test_visual_regression_default_seed() {
    let output = capture_print_output(&["-s", "1"], 80, 24);
    let normalized = normalize_output(&output);
    
    match load_golden("default_seed") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!("Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept.");
            update_golden("default_seed", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_seed_42() {
    let output = capture_print_output(&["-s", "42"], 80, 24);
    let normalized = normalize_output(&output);
    
    match load_golden("seed_42") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!("Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept.");
            update_golden("seed_42", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_seed_123() {
    let output = capture_print_output(&["-s", "123"], 80, 24);
    let normalized = normalize_output(&output);
    
    match load_golden("seed_123") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!("Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept.");
            update_golden("seed_123", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_network_preset() {
    let output = capture_print_output(&["-s", "42", "--preset", "network"], 80, 24);
    let normalized = normalize_output(&output);
    
    match load_golden("network_preset") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!("Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept.");
            update_golden("network_preset", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_exploratory_preset() {
    let output = capture_print_output(&["-s", "42", "--preset", "exploratory"], 80, 24);
    let normalized = normalize_output(&output);
    
    match load_golden("exploratory_preset") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!("Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept.");
            update_golden("exploratory_preset", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_heat_palette() {
    let output = capture_print_output(&["-s", "42", "--palette", "heat"], 80, 24);
    let normalized = normalize_output(&output);
    
    match load_golden("heat_palette") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!("Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept.");
            update_golden("heat_palette", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_ocean_palette() {
    let output = capture_print_output(&["-s", "42", "--palette", "ocean"], 80, 24);
    let normalized = normalize_output(&output);
    
    match load_golden("ocean_palette") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!("Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept.");
            update_golden("ocean_palette", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_ascii_mode() {
    let output = capture_print_output(&["-s", "42", "--ascii"], 80, 24);
    let normalized = normalize_output(&output);
    
    match load_golden("ascii_mode") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!("Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept.");
            update_golden("ascii_mode", &normalized).unwrap();
        }
    }
}

#[test]
fn test_visual_regression_small_terminal() {
    let output = capture_print_output(&["-s", "42"], 40, 12);
    let normalized = normalize_output(&output);
    
    match load_golden("small_terminal") {
        Ok(golden) => {
            assert_eq!(
                normalized, golden,
                "Visual regression: output differs from golden file"
            );
        }
        Err(_) => {
            eprintln!("Warning: Golden file not found, creating it. Run with UPDATE_GOLDEN=1 to accept.");
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
