//! PNG frame export.
//!
//! **Colorize-last invariant**: PNG export colors pixels through the shared
//! [`crate::render::palette::colorize_subpixel`], the SAME entry point used by
//! the live TUI frame buffer and GIF/WebM export. One scalar brightness maps
//! through exactly one colorize pass for every output (TUI, GIF, WebM, PNG), so
//! saved stills match the on-screen look. Do not add a separate
//! palette-mapping path here.

use crate::cli::Palette;
use crate::render::downsample::Cell;
use crate::render::palette;
use crate::render::palette::IntensityMapping;
use image::{Rgb, RgbImage};
use std::time::{SystemTime, UNIX_EPOCH};

/// Maps a single normalized brightness value to an RGB color for PNG export,
/// routing through the shared `colorize_subpixel` colorizer so intensity
/// mapping and temporal-color modulation are applied consistently with the live
/// TUI render.
#[allow(clippy::too_many_arguments)]
fn png_pixel_color(
    norm: f32,
    palette: Palette,
    reverse: bool,
    invert: bool,
    hue_shift: f32,
    mapping: Option<&IntensityMapping>,
    diff_norm: f32,
    temporal_strength: f32,
    temporal_mode: palette::TemporalMode,
    palette_cycle: palette::PaletteCycle,
    temporal_accent: Option<palette::RgbColor>,
) -> palette::RgbColor {
    palette::colorize_subpixel(
        norm,
        palette,
        reverse,
        invert,
        hue_shift,
        mapping,
        diff_norm,
        temporal_strength,
        temporal_mode,
        palette_cycle,
        temporal_accent,
    )
}

/// Saves a single simulation frame as a PNG image.
///
/// Converts the downsampled grid into an image using the specified palette and
/// settings. Each cell renders as two stacked pixels (top/bottom halves), so
/// the image is `width` x `height * 2`. The file is written to the working
/// directory with a timestamp-based name, which is returned.
///
/// `aux_cells` optionally supplies per-cell signed temporal diffs (from
/// [`crate::render::downsample::AuxFrame::cells`]). When provided, temporal
/// color modulation is applied identically to the live TUI render. Pass `None`
/// to disable temporal color in the export.
#[allow(clippy::too_many_arguments)]
pub fn save_frame_as_png(
    downsampled: &[Cell],
    width: usize,
    height: usize,
    palette: Palette,
    reverse_palette: bool,
    invert_palette: bool,
    hue_shift: f32,
    intensity_mapping: Option<&IntensityMapping>,
    max_brightness: f32,
    temporal_strength: f32,
    temporal_mode: palette::TemporalMode,
    aux_cells: Option<&[crate::render::downsample::AuxCell]>,
    palette_cycle: palette::PaletteCycle,
    temporal_accent: Option<palette::RgbColor>,
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

        // Compute normalized signed diff for temporal modulation.
        let diff_norm = if temporal_strength > 0.0 && max_brightness > 0.0 {
            if let Some(aux) = aux_cells {
                if let Some(aux_cell) = aux.get(idx) {
                    aux_cell.signed_diff / max_brightness
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        let top_rgb = png_pixel_color(
            top_norm,
            palette.clone(),
            reverse_palette,
            invert_palette,
            hue_shift,
            intensity_mapping,
            diff_norm,
            temporal_strength,
            temporal_mode,
            palette_cycle,
            temporal_accent,
        );
        let bottom_rgb = png_pixel_color(
            bottom_norm,
            palette.clone(),
            reverse_palette,
            invert_palette,
            hue_shift,
            intensity_mapping,
            diff_norm,
            temporal_strength,
            temporal_mode,
            palette_cycle,
            temporal_accent,
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
    // SystemTime::now() is after UNIX_EPOCH on any modern system, so expect() is safe
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time is before 1970 - this should never happen on modern systems")
        .as_millis();

    // Meant to disambiguate same-millisecond frames, but elapsed() on a fresh
    // Instant is near-zero, so this adds little real uniqueness
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
            None,
            100.0,
            0.0,
            palette::TemporalMode::Hue,
            None,
            palette::PaletteCycle::default(),
            None,
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

        let result = save_frame_as_png(
            &downsampled,
            2,
            1,
            Palette::Heat,
            false,
            false,
            0.0,
            None,
            100.0,
            0.0,
            palette::TemporalMode::Hue,
            None,
            palette::PaletteCycle::default(),
            None,
        );

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

        let result = save_frame_as_png(
            &downsampled,
            2,
            1,
            Palette::Neon,
            true,
            false,
            45.0,
            None,
            100.0,
            0.0,
            palette::TemporalMode::Hue,
            None,
            palette::PaletteCycle::default(),
            None,
        );

        assert!(result.is_ok());

        let filename = result.unwrap();
        let _ = std::fs::remove_file(&filename);
    }

    #[test]
    fn test_png_color_applies_intensity_mapping() {
        use crate::render::palette::{self, IntensityMapping, Palette, TemporalMode};
        let mapping = IntensityMapping::logarithmic(10.0);
        let norm = 0.4_f32;
        let expected = palette::colorize_subpixel(
            norm,
            Palette::Organic,
            false,
            false,
            0.0,
            Some(&mapping),
            0.0,
            0.0,
            TemporalMode::Hue,
            palette::PaletteCycle::default(),
            None,
        );
        let got = png_pixel_color(
            norm,
            Palette::Organic,
            false,
            false,
            0.0,
            Some(&mapping),
            0.0,
            0.0,
            TemporalMode::Hue,
            palette::PaletteCycle::default(),
            None,
        );
        assert_eq!(got, expected);
    }

    #[test]
    fn test_generate_timestamp_format() {
        let filename = generate_timestamp();

        assert!(filename.starts_with("tslime_frame_"));
        assert!(filename.ends_with(".png"));

        let parts: Vec<&str> = filename.split('_').collect();
        // Format: tslime_frame_MILLIS_NANOS.png (4 underscore-separated parts)
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], "tslime");
        assert_eq!(parts[1], "frame");

        // Verify both timestamp parts are numeric
        assert!(parts[2].chars().all(|c| c.is_ascii_digit()));
        let nanos_with_ext = &parts[3];
        let nanos = nanos_with_ext.trim_end_matches(".png");
        assert!(nanos.chars().all(|c| c.is_ascii_digit()));
    }

    /// Guards the colorize-last invariant: the PNG helper must agree with the
    /// shared `colorize_subpixel` entry point across a brightness sweep, a
    /// non-default intensity mapping, and several flag combinations.  Any future
    /// edit to `png_pixel_color` that drops the mapping argument, hardwires a
    /// different palette call, or diverges from `colorize_subpixel` will cause
    /// this test to fail.
    #[test]
    fn test_png_pixel_color_matches_render_colorizer() {
        use crate::render::palette::{self, IntensityMapping, Palette, TemporalMode};

        let mapping = IntensityMapping::logarithmic(10.0);
        let brightnesses = [0.0_f32, 0.15, 0.37, 0.6, 0.83, 1.0];

        // Sweep brightness × {no mapping, with mapping} for default flags.
        for &b in &brightnesses {
            for m in [None, Some(&mapping)] {
                let render = palette::colorize_subpixel(
                    b,
                    Palette::Organic,
                    false,
                    false,
                    0.0,
                    m,
                    0.0,
                    0.0,
                    TemporalMode::Hue,
                    palette::PaletteCycle::default(),
                    None,
                );
                let png = png_pixel_color(
                    b,
                    Palette::Organic,
                    false,
                    false,
                    0.0,
                    m,
                    0.0,
                    0.0,
                    TemporalMode::Hue,
                    palette::PaletteCycle::default(),
                    None,
                );
                assert_eq!(
                    render,
                    png,
                    "PNG/render colorize parity broke at brightness {b} (mapping={})",
                    m.is_some()
                );
            }
        }

        // Also exercise reverse=true and invert=true flag combinations at a
        // representative brightness to catch flag-routing divergence.
        let mid = 0.5_f32;
        for (reverse, invert) in [(true, false), (false, true), (true, true)] {
            let render = palette::colorize_subpixel(
                mid,
                Palette::Organic,
                reverse,
                invert,
                0.0,
                Some(&mapping),
                0.0,
                0.0,
                TemporalMode::Hue,
                palette::PaletteCycle::default(),
                None,
            );
            let png = png_pixel_color(
                mid,
                Palette::Organic,
                reverse,
                invert,
                0.0,
                Some(&mapping),
                0.0,
                0.0,
                TemporalMode::Hue,
                palette::PaletteCycle::default(),
                None,
            );
            assert_eq!(
                render, png,
                "PNG/render colorize parity broke at reverse={reverse} invert={invert}"
            );
        }
    }

    /// Guards temporal-color parity: PNG with temporal_strength > 0 and a non-zero
    /// diff_norm must produce the same color as `colorize_subpixel` called with
    /// the same parameters.
    #[test]
    fn test_png_temporal_color_parity() {
        use crate::render::palette::{self, Palette, TemporalMode};

        let brightnesses = [0.2_f32, 0.5, 0.8];
        let diff_norms = [-0.4_f32, 0.0, 0.4];
        let strengths = [0.0_f32, 0.5, 1.0];

        for &b in &brightnesses {
            for &d in &diff_norms {
                for &s in &strengths {
                    for mode in [TemporalMode::Hue, TemporalMode::Accent] {
                        let render = palette::colorize_subpixel(
                            b,
                            Palette::Organic,
                            false,
                            false,
                            0.0,
                            None,
                            d,
                            s,
                            mode,
                            palette::PaletteCycle::default(),
                            None,
                        );
                        let png = png_pixel_color(
                            b,
                            Palette::Organic,
                            false,
                            false,
                            0.0,
                            None,
                            d,
                            s,
                            mode,
                            palette::PaletteCycle::default(),
                            None,
                        );
                        assert_eq!(
                            render, png,
                            "temporal parity broke at b={b} diff={d} strength={s} mode={mode:?}"
                        );
                    }
                }
            }
        }
    }
}
