use gif::{Encoder, Frame, Repeat};
use std::fs::File;

/// Encodes video frames into an animated GIF file.
pub struct GifExporter {
    width: usize,
    height: usize,
    frames: Vec<Vec<u8>>,
    delay: u16,
}

impl GifExporter {
    /// Creates a new GIF exporter.
    ///
    /// The output path argument is ignored here; pass the path to
    /// [`finish`](Self::finish) instead. `fps` is converted to the GIF frame
    /// delay in hundredths of a second; an fps of 0 falls back to a delay of
    /// 10 (i.e., 10 fps).
    pub fn new(
        width: usize,
        height: usize,
        _output_path: &str,
        fps: usize,
    ) -> Result<Self, String> {
        let delay = if fps > 0 {
            (100.0 / fps as f64).round() as u16
        } else {
            10
        };

        Ok(GifExporter {
            width,
            height,
            frames: Vec::new(),
            delay,
        })
    }

    /// Adds a frame to the animation.
    ///
    /// `pixels` must be a flat RGB byte slice of length `width * height * 3`;
    /// frames of any other length are silently dropped.
    pub fn add_frame_rgb(&mut self, pixels: &[u8]) {
        if pixels.len() == self.width * self.height * 3 {
            self.frames.push(pixels.to_vec());
        }
    }

    /// Finalizes and writes the GIF to the output path.
    pub fn finish(&mut self, output_path: &str) -> Result<(), String> {
        if self.frames.is_empty() {
            return Err("No frames to export".to_string());
        }

        let mut file = File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;

        let mut encoder = Encoder::new(&mut file, self.width as u16, self.height as u16, &[])
            .map_err(|e| format!("Failed to create GIF encoder: {}", e))?;

        encoder
            .set_repeat(Repeat::Infinite)
            .map_err(|e| format!("Failed to set repeat: {}", e))?;

        for frame_data in &self.frames {
            let mut rgba_data = Vec::with_capacity(self.width * self.height * 4);
            for chunk in frame_data.chunks_exact(3) {
                rgba_data.push(chunk[0]);
                rgba_data.push(chunk[1]);
                rgba_data.push(chunk[2]);
                rgba_data.push(255);
            }
            let mut frame = Frame::from_rgba(self.width as u16, self.height as u16, &mut rgba_data);
            frame.delay = self.delay;
            encoder
                .write_frame(&frame)
                .map_err(|e| format!("Failed to write frame: {}", e))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gif_exporter_new() {
        let exporter = GifExporter::new(640, 480, "test.gif", 30);
        assert!(exporter.is_ok());
        let exporter = exporter.unwrap();
        assert_eq!(exporter.width, 640);
        assert_eq!(exporter.height, 480);
    }

    #[test]
    fn test_gif_exporter_new_zero_fps() {
        let exporter = GifExporter::new(640, 480, "test.gif", 0);
        assert!(exporter.is_ok());
        let exporter = exporter.unwrap();
        assert_eq!(exporter.delay, 10);
    }

    #[test]
    fn test_gif_exporter_add_frame() {
        let mut exporter = GifExporter::new(2, 2, "test.gif", 30).unwrap();
        let pixels = vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255]; // RGB for 2x2
        exporter.add_frame_rgb(&pixels);
        assert_eq!(exporter.frames.len(), 1);
    }

    #[test]
    fn test_gif_exporter_add_frame_wrong_size() {
        let mut exporter = GifExporter::new(2, 2, "test.gif", 30).unwrap();
        let wrong_pixels = vec![255, 0, 0]; // Wrong size
        exporter.add_frame_rgb(&wrong_pixels);
        assert_eq!(exporter.frames.len(), 0);
    }

    #[test]
    fn test_gif_exporter_finish_no_frames() {
        let mut exporter = GifExporter::new(2, 2, "test.gif", 30).unwrap();
        let result = exporter.finish("/tmp/nonexistent/test.gif");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No frames to export");
    }

    #[test]
    fn test_gif_exporter_finish_with_frames() {
        let mut exporter = GifExporter::new(2, 2, "test.gif", 30).unwrap();
        let pixels = vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
        exporter.add_frame_rgb(&pixels);

        let output_path = std::env::temp_dir().join("tslime_test.gif");
        let result = exporter.finish(output_path.to_str().unwrap());
        assert!(result.is_ok());

        let _ = std::fs::remove_file(&output_path);
    }

    #[test]
    fn test_gif_exporter_multiple_frames() {
        let mut exporter = GifExporter::new(2, 2, "test.gif", 30).unwrap();
        let pixels = vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];

        for _ in 0..3 {
            exporter.add_frame_rgb(&pixels);
        }

        assert_eq!(exporter.frames.len(), 3);
    }
}
