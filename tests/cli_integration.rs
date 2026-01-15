use std::process::Command;

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tslime"));
    assert!(stdout.to_lowercase().contains("usage"));
}

#[test]
fn test_cli_explain() {
    let output = Command::new("cargo")
        .args(["run", "--", "--explain"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PARAMETER REFERENCE"));
}

#[test]
fn test_cli_invalid_arg() {
    let output = Command::new("cargo")
        .args(["run", "--", "--invalid-argument"])
        .output()
        .expect("failed to execute process");

    assert!(!output.status.success());
}

#[test]
fn test_cli_print_mode() {
    let output = Command::new("cargo")
        .args(["run", "--", "--print", "--init", "random", "-s", "42"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty());
}
