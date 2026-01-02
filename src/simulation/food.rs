use image::io::Reader as ImageReader;
use std::path::Path;

pub fn load_image_grayscale(
    image_path: &str,
    target_width: usize,
    target_height: usize,
    invert: bool,
    scale: f32,
) -> Result<Vec<f32>, String> {
    let path = Path::new(image_path);

    if !path.exists() {
        return Err(format!("Image file not found: {}", image_path));
    }

    let img = ImageReader::open(path)
        .map_err(|e| format!("Failed to open image: {}", e))?
        .decode()
        .map_err(|e| format!("Failed to decode image: {}", e))?;

    let _ = img.width() as usize;
    let _ = img.height() as usize;

    let scaled_width = (target_width as f32 * scale) as usize;
    let scaled_height = (target_height as f32 * scale) as usize;

    let resized = img.resize_exact(
        scaled_width as u32,
        scaled_height as u32,
        image::imageops::FilterType::Nearest,
    );

    let grayscale: Vec<f32> = resized
        .to_luma8()
        .pixels()
        .map(|p| {
            let brightness = p[0] as f32 / 255.0;
            if invert {
                1.0 - brightness
            } else {
                brightness
            }
        })
        .collect();

    let mut result = vec![0.0f32; target_width * target_height];

    let offset_x = (target_width as isize - scaled_width as isize) / 2;
    let offset_y = (target_height as isize - scaled_height as isize) / 2;

    for y in 0..scaled_height {
        for x in 0..scaled_width {
            let src_idx = y * scaled_width + x;
            let dst_x = offset_x + x as isize;
            let dst_y = offset_y + y as isize;

            if dst_x >= 0
                && dst_x < target_width as isize
                && dst_y >= 0
                && dst_y < target_height as isize
            {
                let dst_idx = (dst_y * target_width as isize + dst_x) as usize;
                result[dst_idx] = grayscale[src_idx];
            }
        }
    }

    Ok(result)
}

pub fn get_brightness_at(brightness_map: &[f32], width: usize, x: usize, y: usize) -> f32 {
    if x >= width || y * width + x >= brightness_map.len() {
        return 0.0;
    }
    brightness_map[y * width + x]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_image_grayscale_nonexistent() {
        let result = load_image_grayscale("nonexistent.png", 100, 100, false, 1.0);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_load_image_grayscale_invalid() {
        let result = load_image_grayscale("/dev/null", 100, 100, false, 1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_brightness_at() {
        let map = vec![0.0, 0.5, 1.0, 0.25];
        assert!((get_brightness_at(&map, 2, 0, 0) - 0.0).abs() < 0.001);
        assert!((get_brightness_at(&map, 2, 1, 0) - 0.5).abs() < 0.001);
        assert!((get_brightness_at(&map, 2, 0, 1) - 1.0).abs() < 0.001);
        assert!((get_brightness_at(&map, 2, 1, 1) - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_get_brightness_at_out_of_bounds() {
        let map = vec![0.5, 0.5];
        assert!((get_brightness_at(&map, 2, 5, 0) - 0.0).abs() < 0.001);
        assert!((get_brightness_at(&map, 2, 0, 5) - 0.0).abs() < 0.001);
    }
}
