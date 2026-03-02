use crate::render::palette::RgbColor;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = ".config/tslime";
const PALETTES_FILE: &str = "palettes.toml";

/// A saved palette with name and colors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedPalette {
    /// Name of the palette.
    pub name: String,
    /// Array of 11 RGB colors.
    pub colors: [SerializedColor; 11],
}

/// RGB color for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedColor {
    /// Red component (0-255).
    pub r: u8,
    /// Green component (0-255).
    pub g: u8,
    /// Blue component (0-255).
    pub b: u8,
}

impl From<RgbColor> for SerializedColor {
    fn from(color: RgbColor) -> Self {
        SerializedColor {
            r: color.r,
            g: color.g,
            b: color.b,
        }
    }
}

impl From<SerializedColor> for RgbColor {
    fn from(color: SerializedColor) -> Self {
        RgbColor {
            r: color.r,
            g: color.g,
            b: color.b,
        }
    }
}

impl SavedPalette {
    /// Create a new saved palette from a name and RGB colors.
    pub fn new(name: String, colors: [RgbColor; 11]) -> Self {
        Self {
            name,
            colors: colors.map(|c| c.into()),
        }
    }

    /// Convert the saved palette colors back to RGB.
    pub fn to_rgb_colors(&self) -> [RgbColor; 11] {
        self.colors.clone().map(|c| c.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PalettesFile {
    #[serde(rename = "palette")]
    palettes: Vec<SavedPalette>,
}

fn get_palettes_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "Could not determine home directory".to_string())?;

    let config_dir = PathBuf::from(home).join(CONFIG_DIR);

    fs::create_dir_all(&config_dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    Ok(config_dir.join(PALETTES_FILE))
}

fn load_palettes_file() -> Result<PalettesFile, String> {
    let path = get_palettes_path()?;

    if !path.exists() {
        return Ok(PalettesFile {
            palettes: Vec::new(),
        });
    }

    let contents =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read palettes file: {}", e))?;

    toml::from_str(&contents).map_err(|e| format!("Failed to parse palettes file: {}", e))
}

fn save_palettes_file(file: &PalettesFile) -> Result<(), String> {
    let path = get_palettes_path()?;

    let toml_string =
        toml::to_string_pretty(file).map_err(|e| format!("Failed to serialize palettes: {}", e))?;

    fs::write(&path, toml_string).map_err(|e| format!("Failed to write palettes file: {}", e))
}

/// Save a palette to the palettes file.
pub fn save_palette(palette: SavedPalette) -> Result<(), String> {
    let mut file = load_palettes_file()?;

    file.palettes.retain(|p| p.name != palette.name);

    file.palettes.push(palette);

    save_palettes_file(&file)
}

/// Load a palette by name from the palettes file.
pub fn load_palette(name: &str) -> Result<SavedPalette, String> {
    let file = load_palettes_file()?;

    file.palettes
        .iter()
        .find(|p| p.name == name)
        .cloned()
        .ok_or_else(|| format!("Palette '{}' not found", name))
}

/// List all saved palettes from the palettes file.
pub fn list_palettes() -> Result<Vec<SavedPalette>, String> {
    let file = load_palettes_file()?;
    Ok(file.palettes)
}

/// Delete a palette by name from the palettes file.
pub fn delete_palette(name: &str) -> Result<(), String> {
    let mut file = load_palettes_file()?;

    let original_len = file.palettes.len();
    file.palettes.retain(|p| p.name != name);

    if file.palettes.len() == original_len {
        return Err(format!("Palette '{}' not found", name));
    }

    save_palettes_file(&file)
}

/// Check if a palette with the given name exists.
pub fn palette_exists(name: &str) -> Result<bool, String> {
    let file = load_palettes_file()?;
    Ok(file.palettes.iter().any(|p| p.name == name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_saved_palette_creation() {
        let colors = [
            RgbColor { r: 0, g: 0, b: 0 },
            RgbColor {
                r: 25,
                g: 25,
                b: 25,
            },
            RgbColor {
                r: 50,
                g: 50,
                b: 50,
            },
            RgbColor {
                r: 75,
                g: 75,
                b: 75,
            },
            RgbColor {
                r: 100,
                g: 100,
                b: 100,
            },
            RgbColor {
                r: 125,
                g: 125,
                b: 125,
            },
            RgbColor {
                r: 150,
                g: 150,
                b: 150,
            },
            RgbColor {
                r: 175,
                g: 175,
                b: 175,
            },
            RgbColor {
                r: 200,
                g: 200,
                b: 200,
            },
            RgbColor {
                r: 225,
                g: 225,
                b: 225,
            },
            RgbColor {
                r: 255,
                g: 255,
                b: 255,
            },
        ];

        let saved = SavedPalette::new("TestPalette".to_string(), colors);

        assert_eq!(saved.name, "TestPalette");
        assert_eq!(saved.colors[0].r, 0);
        assert_eq!(saved.colors[10].r, 255);

        let rgb_colors = saved.to_rgb_colors();
        assert_eq!(rgb_colors[5].r, 125);
    }

    #[test]
    fn test_serialized_color_roundtrip() {
        let original = RgbColor {
            r: 128,
            g: 64,
            b: 192,
        };
        let serialized: SerializedColor = original.into();
        let restored: RgbColor = serialized.into();

        assert_eq!(original.r, restored.r);
        assert_eq!(original.g, restored.g);
        assert_eq!(original.b, restored.b);
    }

    #[test]
    fn test_palette_serialization() {
        let colors = [RgbColor {
            r: 100,
            g: 150,
            b: 200,
        }; 11];
        let palette = SavedPalette::new("SerialTest".to_string(), colors);

        let toml_str = toml::to_string(&palette).unwrap();
        let deserialized: SavedPalette = toml::from_str(&toml_str).unwrap();

        assert_eq!(palette.name, deserialized.name);
        assert_eq!(palette.colors[0].r, deserialized.colors[0].r);
    }
}
