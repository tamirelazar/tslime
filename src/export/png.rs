use crate::cli::Palette;
use crate::render::downsample::Cell;
use crate::render::palette;
use image::{Rgb, RgbImage};
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(clippy::too_many_arguments)]
pub fn save_frame_as_png(
    downsampled: &[Cell],
    width: usize,
    height: usize,
    palette: Palette,
    reverse_palette: bool,
    invert_palette: bool,
    hue_shift: f32,
    max_brightness: f32,
) -> Result<String, String> {
    let img_width = width;
    let img_height = height * 2;

    let mut img = RgbImage::new(img_width as u32, img_height as u32);

    for (idx, cell) in downsampled.iter().enumerate() {
        if idx >= width * height {
            break;
        }

        let x = (idx % width) as u32;
        let y = (idx / width) as u32;

        let top_norm = if max_brightness > 0.0 {
            (cell.top / max_brightness).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let bottom_norm = if max_brightness > 0.0 {
            (cell.bottom / max_brightness).clamp(0.0, 1.0)
        } else {
            0.0
        };

        let top_rgb = palette::map_brightness_rgb(
            top_norm,
            palette.clone(),
            reverse_palette,
            invert_palette,
            hue_shift,
        );
        let bottom_rgb = palette::map_brightness_rgb(
            bottom_norm,
            palette.clone(),
            reverse_palette,
            invert_palette,
            hue_shift,
        );

        let top_pixel: Rgb<u8> = Rgb([top_rgb.r, top_rgb.g, top_rgb.b]);
        let bottom_pixel: Rgb<u8> = Rgb([bottom_rgb.r, bottom_rgb.g, bottom_rgb.b]);

        img.put_pixel(x, y * 2, top_pixel);
        img.put_pixel(x, y * 2 + 1, bottom_pixel);
    }

    let filename = generate_timestamp();

    img.save(&filename)
        .map_err(|e| format!("Failed to save PNG: {}", e))?;

    Ok(filename)
}

fn generate_timestamp() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    format!("tslime_frame_{:013}.png", duration.as_millis())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_frame_basic() {
        let downsampled = vec![
            Cell {
                top: 100.0,
                bottom: 50.0,
                ..Default::default()
            },
            Cell {
                top: 75.0,
                bottom: 25.0,
                ..Default::default()
            },
        ];

        let result = save_frame_as_png(
            &downsampled,
            2,
            1,
            Palette::Organic,
            false,
            false,
            0.0,
            100.0,
        );

        assert!(result.is_ok());

        let filename = result.unwrap();
        assert!(filename.starts_with("tslime_frame_"));
        assert!(filename.ends_with(".png"));

        let _ = std::fs::remove_file(&filename);
    }

    #[test]
    fn test_save_frame_with_clamped_brightness() {
        let downsampled = vec![
            Cell {
                top: 150.0,
                bottom: -10.0,
                ..Default::default()
            },
            Cell {
                top: 50.0,
                bottom: 75.0,
                ..Default::default()
            },
        ];

        let result = save_frame_as_png(&downsampled, 2, 1, Palette::Heat, false, false, 0.0, 100.0);

        assert!(result.is_ok());

        let filename = result.unwrap();
        let _ = std::fs::remove_file(&filename);
    }

    #[test]
    fn test_save_frame_with_palette_options() {
        let downsampled = vec![
            Cell {
                top: 50.0,
                bottom: 50.0,
                ..Default::default()
            },
            Cell {
                top: 100.0,
                bottom: 100.0,
                ..Default::default()
            },
        ];

        let result = save_frame_as_png(&downsampled, 2, 1, Palette::Neon, true, false, 45.0, 100.0);

        assert!(result.is_ok());

        let filename = result.unwrap();
        let _ = std::fs::remove_file(&filename);
    }

    #[test]
    fn test_generate_timestamp_format() {
        let filename = generate_timestamp();

        assert!(filename.starts_with("tslime_frame_"));
        assert!(filename.ends_with(".png"));

        let parts: Vec<&str> = filename.split('_').collect();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0], "tslime");
        assert_eq!(parts[1], "frame");

        let timestamp_part = &parts[2];
        let without_ext = timestamp_part.trim_end_matches(".png");
        assert!(without_ext.chars().all(|c| c.is_ascii_digit()));
    }
}
