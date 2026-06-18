use std::process::Command;

/// Path to the compiled `tslime` binary, provided by Cargo for integration
/// tests. Invoking it directly avoids nesting `cargo run` inside `cargo test`,
/// which deadlocks on the build-directory lock (notably on Windows).
fn tslime() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tslime"))
}

/// Asserts the process exited successfully, surfacing the exit code and stderr
/// in the failure message so CI logs show *why* it failed.
fn assert_success(output: &std::process::Output, what: &str) {
    assert!(
        output.status.success(),
        "{what} exited with {:?}\nstderr:\n{}\nstdout:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout),
    );
}

#[test]
fn test_cli_help() {
    let output = tslime()
        .arg("--help")
        .output()
        .expect("failed to execute process");

    assert_success(&output, "--help");
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

    assert_success(&output, "--explain");
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

    assert_success(&output, "--print");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty());
}

#[test]
fn intensity_mapping_absent_is_none_present_is_some() {
    use clap::Parser;
    let bare = tslime::cli::Args::parse_from(["tslime"]);
    assert!(
        bare.intensity_mapping.is_none(),
        "bare run must leave flag unset"
    );

    let explicit = tslime::cli::Args::parse_from(["tslime", "--intensity-mapping", "linear"]);
    assert_eq!(explicit.intensity_mapping.as_deref(), Some("linear"));
}
