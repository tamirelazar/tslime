use gif::{Encoder, Frame, Repeat};
use std::fs::File;

pub struct GifExporter {
    width: usize,
    height: usize,
    frames: Vec<Vec<u8>>,
    delay: u16,
}

impl GifExporter {
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

    pub fn add_frame_rgb(&mut self, pixels: &[u8]) {
        if pixels.len() == self.width * self.height * 3 {
            self.frames.push(pixels.to_vec());
        }
    }

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
