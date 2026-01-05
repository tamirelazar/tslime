use crate::render::palette::RgbColor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridStyle {
    Cross,
    Dots,
    Gradient,
}

impl GridStyle {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "cross" => Ok(GridStyle::Cross),
            "dots" => Ok(GridStyle::Dots),
            "gradient" => Ok(GridStyle::Gradient),
            _ => Err(format!("Unknown grid style: {}", s)),
        }
    }
}

pub struct GridRenderer {
    pub style: GridStyle,
    pub size: usize,
    pub color: RgbColor,
    pub opacity: f32,
    pub adaptive: bool,
}

impl GridRenderer {
    pub fn new(
        style: GridStyle,
        size: usize,
        color: RgbColor,
        opacity: f32,
        adaptive: bool,
    ) -> Self {
        Self {
            style,
            size,
            color,
            opacity,
            adaptive,
        }
    }

    pub fn is_grid_position(&self, x: usize, y: usize, width: usize, height: usize) -> bool {
        if self.size == 0 {
            return false;
        }

        let cell_width = width / self.size;
        let cell_height = height / self.size;

        if cell_width == 0 || cell_height == 0 {
            return false;
        }

        match self.style {
            GridStyle::Cross => {
                // Full horizontal and vertical lines
                x.is_multiple_of(cell_width) || y.is_multiple_of(cell_height)
            }
            GridStyle::Dots => {
                // Only intersections
                x.is_multiple_of(cell_width) && y.is_multiple_of(cell_height)
            }
            GridStyle::Gradient => {
                // Same as cross, but opacity varies based on distance from center
                x.is_multiple_of(cell_width) || y.is_multiple_of(cell_height)
            }
        }
    }

    pub fn calculate_opacity(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
        avg_brightness: f32,
    ) -> f32 {
        let mut opacity = self.opacity;

        // Adaptive opacity - increase when trails are sparse
        if self.adaptive && avg_brightness < 0.2 {
            opacity *= 2.0;
        }

        // Gradient fade from center
        if self.style == GridStyle::Gradient {
            let center_x = width as f32 / 2.0;
            let center_y = height as f32 / 2.0;
            let dx = (x as f32 - center_x).abs() / center_x;
            let dy = (y as f32 - center_y).abs() / center_y;
            let distance = (dx * dx + dy * dy).sqrt();
            let fade = (1.0 - distance * 0.5).max(0.3);
            opacity *= fade;
        }

        opacity.clamp(0.0, 1.0)
    }

    #[allow(dead_code)]
    pub fn blend_color(
        &self,
        grid_color: RgbColor,
        cell_color: RgbColor,
        opacity: f32,
    ) -> RgbColor {
        // Blend: result = cell * (1 - opacity) + grid * opacity
        RgbColor {
            r: ((cell_color.r as f32 * (1.0 - opacity)) + (grid_color.r as f32 * opacity)) as u8,
            g: ((cell_color.g as f32 * (1.0 - opacity)) + (grid_color.g as f32 * opacity)) as u8,
            b: ((cell_color.b as f32 * (1.0 - opacity)) + (grid_color.b as f32 * opacity)) as u8,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_style_from_str() {
        assert_eq!(GridStyle::from_str("cross").unwrap(), GridStyle::Cross);
        assert_eq!(GridStyle::from_str("dots").unwrap(), GridStyle::Dots);
        assert_eq!(GridStyle::from_str("gradient").unwrap(), GridStyle::Gradient);
        assert_eq!(GridStyle::from_str("CROSS").unwrap(), GridStyle::Cross);
        assert!(GridStyle::from_str("invalid").is_err());
    }

    #[test]
    fn test_grid_cross_position() {
        let grid = GridRenderer::new(
            GridStyle::Cross,
            10,
            RgbColor { r: 26, g: 26, b: 26 },
            0.15,
            false,
        );

        // 100x100 grid with size 10 means 10x10 cells
        assert!(grid.is_grid_position(0, 0, 100, 100)); // Corner
        assert!(grid.is_grid_position(10, 0, 100, 100)); // Vertical line
        assert!(grid.is_grid_position(0, 10, 100, 100)); // Horizontal line
        assert!(!grid.is_grid_position(5, 5, 100, 100)); // Not on grid
    }

    #[test]
    fn test_grid_dots_position() {
        let grid = GridRenderer::new(
            GridStyle::Dots,
            10,
            RgbColor { r: 26, g: 26, b: 26 },
            0.15,
            false,
        );

        assert!(grid.is_grid_position(0, 0, 100, 100)); // Intersection
        assert!(!grid.is_grid_position(10, 0, 100, 100)); // Only vertical
        assert!(!grid.is_grid_position(0, 10, 100, 100)); // Only horizontal
        assert!(!grid.is_grid_position(5, 5, 100, 100)); // Not on grid
    }

    #[test]
    fn test_adaptive_opacity() {
        let grid = GridRenderer::new(
            GridStyle::Cross,
            10,
            RgbColor { r: 26, g: 26, b: 26 },
            0.15,
            true,
        );

        // Low brightness should increase opacity
        let low_opacity = grid.calculate_opacity(0, 0, 100, 100, 0.1);
        assert!(low_opacity > 0.15);

        // High brightness should keep normal opacity
        let high_opacity = grid.calculate_opacity(0, 0, 100, 100, 0.5);
        assert_eq!(high_opacity, 0.15);
    }

    #[test]
    fn test_color_blending() {
        let grid = GridRenderer::new(
            GridStyle::Cross,
            10,
            RgbColor { r: 26, g: 26, b: 26 },
            0.5,
            false,
        );

        let grid_color = RgbColor { r: 100, g: 100, b: 100 };
        let cell_color = RgbColor { r: 200, g: 200, b: 200 };
        let blended = grid.blend_color(grid_color, cell_color, 0.5);

        // Should be 50/50 blend
        assert_eq!(blended.r, 150);
        assert_eq!(blended.g, 150);
        assert_eq!(blended.b, 150);
    }
}
