//! Loads user-defined key bindings from `~/.config/tslime/keybinds.toml`.
//!
//! Keys `1`-`7` may each bind a preset OR a saved config. Invalid entries are
//! dropped silently; a missing/unreadable/malformed file yields no binds, so the
//! app always launches on the built-in defaults. Mirrors `palette_manager`.

use crate::simulation::config::{preset_from_name, Preset};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

const CONFIG_DIR: &str = ".config/tslime";
const KEYBINDS_FILE: &str = "keybinds.toml";

/// What a bound key does when pressed.
#[derive(Debug, Clone, PartialEq)]
pub enum BindTarget {
    /// Switch to this preset.
    Preset(Preset),
    /// Load this saved config by name.
    Config(String),
}

#[derive(Debug, Deserialize)]
struct RawKeybind {
    key: String,
    preset: Option<String>,
    config: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct RawFile {
    #[serde(default)]
    keybind: Vec<RawKeybind>,
}

fn keybinds_path() -> Option<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()?;
    Some(PathBuf::from(home).join(CONFIG_DIR).join(KEYBINDS_FILE))
}

/// Parse + validate raw entries into a resolved map (last valid entry per key
/// wins).
fn parse_entries(entries: Vec<RawKeybind>, known_configs: &[String]) -> HashMap<char, BindTarget> {
    let mut map = HashMap::new();
    for e in entries {
        let mut chars = e.key.chars();
        let key = match (chars.next(), chars.next()) {
            (Some(c), None) if ('1'..='7').contains(&c) => c,
            _ => continue,
        };
        let target = match (e.preset.as_deref(), e.config.as_deref()) {
            (Some(p), None) => match preset_from_name(p) {
                Some(preset) => BindTarget::Preset(preset),
                None => continue,
            },
            (None, Some(c)) => {
                if let Some(canonical) = known_configs.iter().find(|n| n.eq_ignore_ascii_case(c)) {
                    BindTarget::Config(canonical.clone())
                } else {
                    continue;
                }
            }
            _ => continue, // both set or neither set: invalid
        };
        map.insert(key, target); // later valid entry overrides earlier
    }
    map
}

/// Parse keybinds from TOML text. Malformed TOML yields an empty map.
pub(crate) fn load_from_str(contents: &str, known_configs: &[String]) -> HashMap<char, BindTarget> {
    match toml::from_str::<RawFile>(contents) {
        Ok(raw) => parse_entries(raw.keybind, known_configs),
        Err(_) => HashMap::new(),
    }
}

/// Load + resolve keybinds from disk. Absent/unreadable/malformed file →
/// empty map.
/// If the saved-config list can't be enumerated, config binds are dropped but
/// preset binds still load (known_configs is empty).
#[must_use]
pub fn load_keybinds() -> HashMap<char, BindTarget> {
    let path = match keybinds_path() {
        Some(p) => p,
        None => return HashMap::new(),
    };
    let contents = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    let known_configs = crate::config_manager::list_configs()
        .map(|v| v.into_iter().map(|c| c.name).collect::<Vec<_>>())
        .unwrap_or_default();
    load_from_str(&contents, &known_configs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::config::Preset;

    fn cfgs() -> Vec<String> {
        vec!["mynight".to_string()]
    }

    #[test]
    fn valid_preset_and_config_binds_load() {
        let toml = r#"
            [[keybind]]
            key = "4"
            preset = "fire"
            [[keybind]]
            key = "5"
            config = "mynight"
        "#;
        let m = load_from_str(toml, &cfgs());
        assert_eq!(m.get(&'4'), Some(&BindTarget::Preset(Preset::Fire)));
        assert_eq!(
            m.get(&'5'),
            Some(&BindTarget::Config("mynight".to_string()))
        );
    }

    #[test]
    fn invalid_entries_are_dropped() {
        let toml = r#"
            [[keybind]]
            key = "8"
            preset = "fire"
            [[keybind]]
            key = "4"
            preset = "no-such-preset"
            [[keybind]]
            key = "5"
            config = "no-such-config"
            [[keybind]]
            key = "6"
            preset = "fire"
            config = "mynight"
            [[keybind]]
            key = "7"
        "#;
        let m = load_from_str(toml, &cfgs());
        assert!(
            m.is_empty(),
            "out-of-pool, unknown, both, and neither all drop"
        );
    }

    #[test]
    fn malformed_toml_yields_empty() {
        assert!(load_from_str("= = not toml", &cfgs()).is_empty());
    }

    #[test]
    fn duplicate_last_valid_wins() {
        let m = load_from_str(
            "[[keybind]]\nkey=\"4\"\npreset=\"fire\"\n[[keybind]]\nkey=\"4\"\npreset=\"organic\"\n",
            &cfgs(),
        );
        assert_eq!(m.get(&'4'), Some(&BindTarget::Preset(Preset::Organic)));

        let m = load_from_str(
            "[[keybind]]\nkey=\"4\"\npreset=\"fire\"\n[[keybind]]\nkey=\"4\"\npreset=\"nope\"\n",
            &cfgs(),
        );
        assert_eq!(m.get(&'4'), Some(&BindTarget::Preset(Preset::Fire)));

        let m = load_from_str(
            "[[keybind]]\nkey=\"4\"\npreset=\"nope\"\n[[keybind]]\nkey=\"4\"\npreset=\"fire\"\n",
            &cfgs(),
        );
        assert_eq!(m.get(&'4'), Some(&BindTarget::Preset(Preset::Fire)));
    }

    #[test]
    fn unknown_config_dropped_when_no_configs() {
        let m = load_from_str("[[keybind]]\nkey=\"4\"\nconfig=\"mynight\"\n", &[]);
        assert!(m.is_empty());
    }

    #[test]
    fn config_casing_stored_canonically() {
        let known = vec!["mynight".to_string()];
        let m = load_from_str("[[keybind]]\nkey=\"4\"\nconfig=\"MYNIGHT\"\n", &known);
        assert_eq!(
            m.get(&'4'),
            Some(&BindTarget::Config("mynight".to_string())),
            "config name stored as canonical, not user input"
        );
    }
}
