use rand::Rng;
use std::path::PathBuf;
use std::process::Command;

pub struct WebmExporter {
    width: usize,
    height: usize,
    frames: Vec<PathBuf>,
    fps: usize,
    temp_dir: PathBuf,
}

impl WebmExporter {
    pub fn new(
        width: usize,
        height: usize,
        _output_path: &str,
        fps: usize,
    ) -> Result<Self, String> {
        let mut rng = rand::thread_rng();
        let temp_dir = PathBuf::from(format!(
            "webm_frames_{}_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis(),
            rng.gen::<u32>()
        ));

        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp directory: {}", e))?;

        Ok(WebmExporter {
            width,
            height,
            frames: Vec::new(),
            fps,
            temp_dir,
        })
    }

    pub fn add_frame_png(&mut self, pixels: &[u8]) -> Result<(), String> {
        let frame_idx = self.frames.len();
        let filename = self.temp_dir.join(format!("frame_{:04}.png", frame_idx));

        image::save_buffer(
            &filename,
            pixels,
            self.width as u32,
            self.height as u32,
            image::ColorType::Rgb8,
        )
        .map_err(|e| format!("Failed to save frame: {}", e))?;

        self.frames.push(filename);
        Ok(())
    }

    pub fn finish(&mut self, output_path: &str) -> Result<(), String> {
        if self.frames.is_empty() {
            return Err("No frames to export".to_string());
        }

        let frame_pattern = self.temp_dir.join("frame_%04d.png");

        eprintln!("Encoding WebM video with FFmpeg...");

        let frame_pattern_str = frame_pattern
            .to_str()
            .ok_or_else(|| "Frame pattern path contains invalid UTF-8".to_string())?;

        #[allow(clippy::needless_borrows_for_generic_args)]
        let status = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-framerate",
                &self.fps.to_string(),
                "-i",
                frame_pattern_str,
                "-c:v",
                "libvpx-vp9",
                "-lossless",
                "1",
                "-pix_fmt",
                "yuva420p",
                output_path,
            ])
            .status()
            .map_err(|e| format!("Failed to run FFmpeg: {}", e))?;

        if !status.success() {
            return Err("FFmpeg failed to encode WebM".to_string());
        }

        self.cleanup()?;

        Ok(())
    }

    fn cleanup(&mut self) -> Result<(), String> {
        for frame in &self.frames {
            if let Err(e) = std::fs::remove_file(frame) {
                eprintln!(
                    "Warning: Failed to remove temp frame {}: {}",
                    frame.display(),
                    e
                );
            }
        }

        if let Err(e) = std::fs::remove_dir(&self.temp_dir) {
            eprintln!(
                "Warning: Failed to remove temp directory {}: {}",
                self.temp_dir.display(),
                e
            );
        }

        self.frames.clear();
        Ok(())
    }
}

impl Drop for WebmExporter {
    fn drop(&mut self) {
        if !self.frames.is_empty() {
            let _ = self.cleanup();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webm_exporter_new() {
        let exporter = WebmExporter::new(640, 480, "test.webm", 30);
        assert!(exporter.is_ok());
        let exporter = exporter.unwrap();
        assert_eq!(exporter.width, 640);
        assert_eq!(exporter.height, 480);
        assert_eq!(exporter.fps, 30);
    }

    #[test]
    fn test_webm_exporter_temp_dir_created() {
        let exporter = WebmExporter::new(640, 480, "test.webm", 30).unwrap();
        assert!(exporter.temp_dir.exists());
        assert!(exporter.temp_dir.to_string_lossy().contains("webm_frames_"));
        let _ = std::fs::remove_dir_all(&exporter.temp_dir);
    }

    #[test]
    fn test_webm_exporter_add_frame() {
        let mut exporter = WebmExporter::new(2, 2, "test.webm", 30).unwrap();
        let pixels = vec![255u8; 12]; // 2x2 RGB = 12 bytes
        let result = exporter.add_frame_png(&pixels);
        assert!(result.is_ok());
        // Don't check frame count here as Drop may have cleaned up
    }

    #[test]
    fn test_webm_exporter_multiple_frames() {
        let mut exporter = WebmExporter::new(2, 2, "test.webm", 30).unwrap();
        let pixels = vec![255u8; 12];

        for i in 0..5 {
            let result = exporter.add_frame_png(&pixels);
            assert!(result.is_ok(), "Failed to add frame {}", i);
        }
        // Verify frames were added before Drop cleans up
        assert_eq!(exporter.frames.len(), 5);
    }

    #[test]
    fn test_webm_exporter_cleanup_removes_frames() {
        let mut exporter = WebmExporter::new(2, 2, "test.webm", 30).unwrap();
        let pixels = vec![255u8; 12];
        let _ = exporter.add_frame_png(&pixels);

        // Verify frames exist before cleanup
        assert!(!exporter.frames.is_empty());

        let temp_dir_path = exporter.temp_dir.clone();
        assert!(temp_dir_path.exists());

        let _ = exporter.cleanup();
        assert!(!temp_dir_path.exists());
    }
}
