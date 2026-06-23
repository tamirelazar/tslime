//! Byte-identity net for the apply()-collapse refactor: dumps every preset's
//! resolved `SimConfig` (preset-only, no CLI overrides) to a golden file.
//! Excludes afterglow (it leaves SimConfig unchanged after Phase B commit 2).
//! 27/29 presets must stay byte-identical through the collapse;
//! River + PetriDish change deliberately (commit 1) and are reviewed in the diff.

use std::process::{Command, Stdio};

const GOLDEN: &str = "tests/golden/preset_configs.txt";

/// All preset CLI tokens, in PRESETS order (src/simulation/config.rs:144-319).
const PRESETS: &[&str] = &[
    "network",
    "exploratory",
    "tendrils",
    "organic",
    "fire",
    "river",
    "petridish",
    "vortex",
    "lightning",
    "chaosedge",
    "blob",
    "slime",
    "vines",
    "vinescii",
    "smoke",
    "vortex36",
    "dynamictendrils",
    "mold",
    "etching",
    "drift",
    "constellations",
    "mosaic",
    "marble",
    "prism",
    "vellum",
    "forge",
    "wane",
    "gossamer",
    "codex",
    "tide",
];

/// Dump the assembled config for `preset` by invoking the binary's hidden
/// `--dump-config` mode (added in Step 2), which prints the curated field set.
fn dump_assembled(preset: &str) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_tslime"))
        .args(["--dump-config", "--preset", preset])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn tslime --dump-config");
    assert!(
        out.status.success(),
        "tslime --dump-config --preset {preset} failed ({}): {}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8 dump")
}

#[test]
fn preset_assembled_configs_match_golden() {
    let mut buf = String::new();
    for p in PRESETS {
        buf.push_str(&format!("=== {p} ===\n"));
        buf.push_str(&dump_assembled(p));
        buf.push('\n');
    }
    if std::env::var("UPDATE_GOLDEN").is_ok() {
        std::fs::write(GOLDEN, &buf).expect("write golden");
        return;
    }
    let golden = std::fs::read_to_string(GOLDEN).unwrap_or_default();
    assert_eq!(buf, golden, "assembled preset configs differ from golden");
}
