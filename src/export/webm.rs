use std::fs::File;
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
        let temp_dir = PathBuf::from(format!(
            "webm_frames_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
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

        let file =
            File::create(&filename).map_err(|e| format!("Failed to create frame file: {}", e))?;

        let encoder = png::Encoder::new(file, self.width as u32, self.height as u32);
        let mut writer = encoder
            .write_header()
            .map_err(|e| format!("Failed to write PNG header: {}", e))?;
        writer
            .write_image_data(pixels)
            .map_err(|e| format!("Failed to write PNG data: {}", e))?;

        self.frames.push(filename);
        Ok(())
    }

    pub fn finish(&mut self, output_path: &str) -> Result<(), String> {
        if self.frames.is_empty() {
            return Err("No frames to export".to_string());
        }

        let frame_pattern = self.temp_dir.join("frame_%04d.png");

        eprintln!("Encoding WebM video with FFmpeg...");

        #[allow(clippy::needless_borrows_for_generic_args)]
        let status = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-framerate",
                &self.fps.to_string(),
                "-i",
                frame_pattern.to_str().unwrap(),
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

    #[allow(dead_code)]
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}

impl Drop for WebmExporter {
    fn drop(&mut self) {
        if !self.frames.is_empty() {
            let _ = self.cleanup();
        }
    }
}
