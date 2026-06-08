use std::process::Command;

/// Path to the compiled `tslime` binary, provided by Cargo for integration
/// tests. Invoking it directly avoids nesting `cargo run` inside `cargo test`,
/// which deadlocks on the build-directory lock (notably on Windows).
fn tslime() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tslime"))
}

#[test]
fn test_cli_help() {
    let output = tslime()
        .arg("--help")
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("tslime"));
    assert!(stdout.to_lowercase().contains("usage"));
}

#[test]
fn test_cli_explain() {
    let output = tslime()
        .arg("--explain")
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("PARAMETER REFERENCE"));
}

#[test]
fn test_cli_invalid_arg() {
    let output = tslime()
        .arg("--invalid-argument")
        .output()
        .expect("failed to execute process");

    assert!(!output.status.success());
}

#[test]
fn test_cli_print_mode() {
    let output = tslime()
        .args(["--print", "--init", "random", "-s", "42"])
        .output()
        .expect("failed to execute process");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty());
}
