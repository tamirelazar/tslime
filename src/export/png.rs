use crate::cli::Palette;
use crate::render::downsample::Cell;
use crate::render::palette;
use image::{Rgb, RgbImage};
use std::time::{SystemTime, UNIX_EPOCH};

/// Saves a single simulation frame as a PNG image.
///
/// Converts the downsampled grid into an image using the specified palette and settings.
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
            None,
        );
        let bottom_rgb = palette::map_brightness_rgb(
            bottom_norm,
            palette.clone(),
            reverse_palette,
            invert_palette,
            hue_shift,
            None,
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
    // Get current time since UNIX_EPOCH
    // SystemTime::now() is guaranteed to be after UNIX_EPOCH on all modern systems
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time is before 1970 - this should never happen on modern systems")
        .as_millis();

    // Add nanoseconds for additional uniqueness to avoid collisions
    let nanos = std::time::Instant::now().elapsed().subsec_nanos();
    format!("tslime_frame_{:013}_{:09}.png", millis, nanos)
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
        // New format: tslime_frame_MILLIS_NANOS.png (4 parts)
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], "tslime");
        assert_eq!(parts[1], "frame");

        // Verify both timestamp parts are numeric
        assert!(parts[2].chars().all(|c| c.is_ascii_digit()));
        let nanos_with_ext = &parts[3];
        let nanos = nanos_with_ext.trim_end_matches(".png");
        assert!(nanos.chars().all(|c| c.is_ascii_digit()));
    }
}
