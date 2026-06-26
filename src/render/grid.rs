use crate::render::palette::RgbColor;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
/// Visual style of the background grid.
pub enum GridStyle {
    /// Solid lines intersecting at grid points.
    Cross,
    /// Points only at intersections.
    Dots,
    /// Lines that fade out from the center.
    Gradient,
}

impl FromStr for GridStyle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "cross" => Ok(GridStyle::Cross),
            "dots" => Ok(GridStyle::Dots),
            "gradient" => Ok(GridStyle::Gradient),
            _ => Err(format!("Unknown grid style: {}", s)),
        }
    }
}

#[derive(Clone)]
/// Renders a background grid to provide spatial reference.
pub struct GridRenderer {
    /// Rendering style.
    pub style: GridStyle,
    /// Number of grid cells per axis; line spacing is derived from terminal size.
    pub size: usize,
    /// Grid line color.
    pub color: RgbColor,
    /// Base opacity of the grid.
    pub opacity: f32,
    /// Whether opacity adapts to content brightness.
    pub adaptive: bool,
    col_positions: Vec<usize>,
    row_positions: Vec<usize>,
}

impl GridRenderer {
    /// Creates a new grid renderer.
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
            col_positions: Vec::new(),
            row_positions: Vec::new(),
        }
    }

    /// Calculate grid line positions with center-heavy remainder distribution.
    /// When terminal_size % grid_size != 0, the remainder is distributed to the center-most lines.
    ///
    /// Example: 24 rows, grid_size 10:
    /// - Base spacing: 24 / 10 = 2
    /// - Remainder: 24 % 10 = 4
    /// - Result: 2-2-2-3-3-3-3-2-2-2 (center 4 cells get +1 spacing)
    fn calculate_grid_positions(terminal_size: usize, grid_size: usize) -> Vec<usize> {
        if grid_size == 0 || terminal_size == 0 {
            return Vec::new();
        }

        let base_spacing = terminal_size / grid_size;
        let remainder = terminal_size % grid_size;

        if base_spacing == 0 {
            return Vec::new();
        }

        let mut positions = Vec::with_capacity(grid_size - 1);
        let mut current_pos = 0;

        // Center-heavy distribution: the middle `remainder` cells get +1 spacing
        let start_extra = (grid_size - remainder) / 2;
        let end_extra = start_extra + remainder;

        for i in 0..grid_size {
            let cell_height = if i >= start_extra && i < end_extra {
                base_spacing + 1
            } else {
                base_spacing
            };

            current_pos += cell_height;

            // Add line position (but not after the last cell)
            if i < grid_size - 1 {
                positions.push(current_pos);
            }
        }

        positions
    }

    /// Initialize grid positions based on terminal dimensions.
    /// Must be called before is_grid_position().
    pub fn initialize(&mut self, width: usize, height: usize) {
        self.col_positions = Self::calculate_grid_positions(width, self.size);
        self.row_positions = Self::calculate_grid_positions(height, self.size);
    }

    /// Checks if a pixel coordinate lies on a grid line or intersection.
    pub fn is_grid_position(&self, x: usize, y: usize, _width: usize, _height: usize) -> bool {
        let on_vertical = self.col_positions.contains(&x);
        let on_horizontal = self.row_positions.contains(&y);

        match self.style {
            GridStyle::Cross => {
                // Full horizontal and vertical lines
                on_vertical || on_horizontal
            }
            GridStyle::Dots => {
                // Only intersections
                on_vertical && on_horizontal
            }
            GridStyle::Gradient => {
                // Same as cross, but opacity varies based on distance from center
                on_vertical || on_horizontal
            }
        }
    }

    /// Returns (on_vertical, on_horizontal) for a given position
    pub fn get_grid_lines(&self, x: usize, y: usize) -> (bool, bool) {
        let on_vertical = self.col_positions.contains(&x);
        let on_horizontal = self.row_positions.contains(&y);
        (on_vertical, on_horizontal)
    }

    /// Calculates the effective opacity for a grid pixel.
    ///
    /// Accounts for adaptive opacity (based on content brightness) and gradient style.
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

    /// Blends the grid color with the underlying cell color.
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
        assert_eq!(
            GridStyle::from_str("gradient").unwrap(),
            GridStyle::Gradient
        );
        assert_eq!(GridStyle::from_str("CROSS").unwrap(), GridStyle::Cross);
        assert!(GridStyle::from_str("invalid").is_err());
    }

    #[test]
    fn test_grid_positions_exact_division() {
        // 30 rows, grid_size 10 → 3 per cell, lines at 3, 6, 9, 12, 15, 18, 21, 24, 27
        let positions = GridRenderer::calculate_grid_positions(30, 10);
        assert_eq!(positions.len(), 9);
        assert_eq!(positions[0], 3);
        assert_eq!(positions[1], 6);
        assert_eq!(positions[2], 9);
        // All spacing should be 3
        for i in 1..positions.len() {
            assert_eq!(positions[i] - positions[i - 1], 3);
        }
    }

    #[test]
    fn test_grid_positions_with_remainder() {
        // 24 rows, grid_size 10 → base spacing 2, remainder 4
        // Should distribute 4 extra spaces to center cells
        // Cell heights: 2, 2, 2, 3, 3, 3, 3, 2, 2, 2
        // Line positions: 2, 4, 6, 9, 12, 15, 18, 20, 22
        let positions = GridRenderer::calculate_grid_positions(24, 10);
        assert_eq!(positions.len(), 9);

        // Verify the actual positions
        assert_eq!(positions[0], 2); // After first cell (height 2)
        assert_eq!(positions[1], 4); // After second cell (height 2)
        assert_eq!(positions[2], 6); // After third cell (height 2)
        assert_eq!(positions[3], 9); // After fourth cell (height 3) - center starts
        assert_eq!(positions[4], 12); // After fifth cell (height 3)
        assert_eq!(positions[5], 15); // After sixth cell (height 3)
        assert_eq!(positions[6], 18); // After seventh cell (height 3) - center ends
        assert_eq!(positions[7], 20); // After eighth cell (height 2)
        assert_eq!(positions[8], 22); // After ninth cell (height 2)
    }

    #[test]
    fn test_grid_positions_edge_cases() {
        // Zero grid size
        let positions = GridRenderer::calculate_grid_positions(100, 0);
        assert_eq!(positions.len(), 0);

        // Zero terminal size
        let positions = GridRenderer::calculate_grid_positions(0, 10);
        assert_eq!(positions.len(), 0);

        // Grid size larger than terminal
        let positions = GridRenderer::calculate_grid_positions(5, 10);
        assert_eq!(positions.len(), 0);

        // Grid size = 1 (single cell, no lines)
        let positions = GridRenderer::calculate_grid_positions(100, 1);
        assert_eq!(positions.len(), 0);
    }

    #[test]
    fn test_grid_cross_position() {
        let mut grid = GridRenderer::new(
            GridStyle::Cross,
            10,
            RgbColor {
                r: 255,
                g: 255,
                b: 255,
            },
            0.15,
            false,
        );
        grid.initialize(100, 100);

        // 100x100 grid with size 10 means spacing of 10
        // Lines at: 10, 20, 30, 40, 50, 60, 70, 80, 90
        assert!(grid.is_grid_position(10, 0, 100, 100)); // Vertical line
        assert!(grid.is_grid_position(0, 10, 100, 100)); // Horizontal line
        assert!(grid.is_grid_position(10, 10, 100, 100)); // Intersection
        assert!(!grid.is_grid_position(5, 5, 100, 100)); // Not on grid
        assert!(!grid.is_grid_position(0, 0, 100, 100)); // Corner is not a line
    }

    #[test]
    fn test_grid_dots_position() {
        let mut grid = GridRenderer::new(
            GridStyle::Dots,
            10,
            RgbColor {
                r: 255,
                g: 255,
                b: 255,
            },
            0.15,
            false,
        );
        grid.initialize(100, 100);

        // Dots only at intersections
        assert!(grid.is_grid_position(10, 10, 100, 100)); // Intersection
        assert!(!grid.is_grid_position(10, 0, 100, 100)); // Only vertical
        assert!(!grid.is_grid_position(0, 10, 100, 100)); // Only horizontal
        assert!(!grid.is_grid_position(5, 5, 100, 100)); // Not on grid
    }

    #[test]
    fn test_adaptive_opacity() {
        let grid = GridRenderer::new(
            GridStyle::Cross,
            10,
            RgbColor {
                r: 26,
                g: 26,
                b: 26,
            },
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
            RgbColor {
                r: 26,
                g: 26,
                b: 26,
            },
            0.5,
            false,
        );

        let grid_color = RgbColor {
            r: 100,
            g: 100,
            b: 100,
        };
        let cell_color = RgbColor {
            r: 200,
            g: 200,
            b: 200,
        };
        let blended = grid.blend_color(grid_color, cell_color, 0.5);

        // Should be 50/50 blend
        assert_eq!(blended.r, 150);
        assert_eq!(blended.g, 150);
        assert_eq!(blended.b, 150);
    }
}
