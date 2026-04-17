//! Runtime state and control types.
//!
//! This module defines all the state types used for controlling the simulation,
//! including RuntimeState, ControlAction, and various parameter types.

pub use crate::cli::num_palettes;
use crate::cli::Palette;
use crate::cli::PauseStyle;
pub use crate::cli::ALL_PALETTES;
pub use crate::cli::NUM_PALETTES;
use crate::config_defaults::dither as dither_consts;
use crate::config_defaults::intensity::DEFAULT_PERLIN_SEED as PERLIN_SEED;
use crate::config_defaults::{agent as agent_consts, environment as env_consts, trail};
use crate::food_image::FOOD_IMAGE_PNG;
use crate::overlay::{OverlayState, OverlayType};
use crate::render::charset::Charset;
pub use crate::render::charset::ALL_CHARSETS;
use crate::render::dither::{DitherMatrix, DitherMode};
use crate::render::options_overlay::ControlsOverlay;
use crate::render::palette::IntensityMapping;
use crate::render::theme::{PanelStyle, ALL_THEMES, GRUVBOX_DARK};
use crate::simulation::config::{
    Aspect, ChromeStyle, DiffusionKernel, InitMode, Preset, SimConfig, TerminalSizeThreshold,
    TerrainType, Wind, WindowFrame, WindowPadding,
};
use crate::simulation::food::load_logo_from_memory;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
/// Represents a mouse cursor position in the terminal grid.
pub struct MousePosition {
    /// X coordinate (column).
    pub x: usize,
    /// Y coordinate (row).
    pub y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Defines how mouse interaction affects the simulation.
pub enum MouseInteractionMode {
    /// Mouse interaction disabled.
    Disabled,
    /// Mouse click creates an attractor.
    Attract,
    /// Mouse click creates a repeller.
    Repel,
}

/// Returns the number of available charsets.
pub fn num_charsets() -> usize {
    ALL_CHARSETS.len()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Speed of automatic palette hue shifting.
pub enum PaletteShiftSpeed {
    /// No shift.
    Off,
    /// Slow shift (5 degrees/sec).
    Slow,
    /// Medium shift (15 degrees/sec).
    Medium,
    /// Fast shift (45 degrees/sec).
    Fast,
}

impl PaletteShiftSpeed {
    /// Returns the shift speed in degrees per second.
    pub fn degrees_per_second(&self) -> f32 {
        match self {
            PaletteShiftSpeed::Off => 0.0,
            PaletteShiftSpeed::Slow => 5.0,
            PaletteShiftSpeed::Medium => 15.0,
            PaletteShiftSpeed::Fast => 45.0,
        }
    }
}

/// Urgency level for toast notifications.
///
/// Controls the icon prefix and accent color of a notification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NotificationLevel {
    /// Neutral informational message (teal accent).
    Info,
    /// Positive confirmation (green accent).
    Success,
    /// Non-critical caution (amber accent).
    Warning,
    /// Critical error (red accent).
    Error,
}

impl NotificationLevel {
    /// Returns a single-character icon prefix for the notification.
    pub fn icon(self) -> &'static str {
        match self {
            NotificationLevel::Info => "ℹ",
            NotificationLevel::Success => "✓",
            NotificationLevel::Warning => "⚠",
            NotificationLevel::Error => "✗",
        }
    }

    /// Returns the ANSI 256-color index for this level's accent background.
    pub fn bg_color_256(self) -> u8 {
        match self {
            NotificationLevel::Info => 23,    // Dark teal
            NotificationLevel::Success => 22, // Dark green
            NotificationLevel::Warning => 94, // Dark amber/orange
            NotificationLevel::Error => 52,   // Dark red
        }
    }

    /// Returns an RGB accent color for this level.
    pub fn accent_rgb(self) -> crate::render::palette::RgbColor {
        use crate::render::palette::RgbColor;
        match self {
            NotificationLevel::Info => RgbColor {
                r: 69,
                g: 192,
                b: 191,
            }, // aqua
            NotificationLevel::Success => RgbColor {
                r: 142,
                g: 192,
                b: 124,
            }, // green
            NotificationLevel::Warning => RgbColor {
                r: 215,
                g: 153,
                b: 33,
            }, // amber
            NotificationLevel::Error => RgbColor {
                r: 251,
                g: 73,
                b: 52,
            }, // red bright
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Predefined wind directions for easy control.
pub enum WindDirection {
    /// No wind.
    None,
    /// Wind blowing North (up).
    North,
    /// Wind blowing Northeast.
    Northeast,
    /// Wind blowing East (right).
    East,
    /// Wind blowing Southeast.
    Southeast,
    /// Wind blowing South (down).
    South,
    /// Wind blowing Southwest.
    Southwest,
    /// Wind blowing West (left).
    West,
    /// Wind blowing Northwest.
    Northwest,
}

impl WindDirection {
    #[allow(clippy::wrong_self_convention)]
    /// Converts the enum variant to a `Wind` vector.
    pub fn to_wind(&self) -> Option<Wind> {
        match self {
            WindDirection::None => None,
            WindDirection::North => Some(Wind::new(0.0, -1.0)),
            WindDirection::Northeast => Some(Wind::new(0.7, -0.7)),
            WindDirection::East => Some(Wind::new(1.0, 0.0)),
            WindDirection::Southeast => Some(Wind::new(0.7, 0.7)),
            WindDirection::South => Some(Wind::new(0.0, 1.0)),
            WindDirection::Southwest => Some(Wind::new(-0.7, 0.7)),
            WindDirection::West => Some(Wind::new(-1.0, 0.0)),
            WindDirection::Northwest => Some(Wind::new(-0.7, -0.7)),
        }
    }

    /// Returns the display name of the direction.
    pub fn name(&self) -> &'static str {
        match self {
            WindDirection::None => "None",
            WindDirection::North => "N",
            WindDirection::Northeast => "NE",
            WindDirection::East => "E",
            WindDirection::Southeast => "SE",
            WindDirection::South => "S",
            WindDirection::Southwest => "SW",
            WindDirection::West => "W",
            WindDirection::Northwest => "NW",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Actions triggered by keyboard or other input events.
pub enum ControlAction {
    /// Pause/resume simulation.
    TogglePause,
    /// Restart simulation with new seed.
    Restart,
    /// Apply a preset configuration.
    SetPreset(Preset),
    /// Show preset comparison overlay.
    ComparePreset(Preset),
    /// Adjust simulation speed.
    AdjustTimeScale(f32),
    /// Cycle to next color palette.
    CyclePalette,
    /// Cycle to previous color palette.
    CyclePaletteReverse,
    /// Cycle to next charset.
    CycleCharset,
    /// Cycle to previous charset.
    CycleCharsetReverse,
    /// Toggle controls overlay.
    ToggleControls,
    /// Toggle keyboard shortcuts overlay.
    ToggleKeyboardHints,
    /// Close all open overlays.
    CloseOverlays,
    /// Toggle dithering on/off.
    ToggleDither,
    /// Cycle through dithering modes.
    CycleDitherMode,
    /// Adjust dithering intensity.
    AdjustDitherIntensity(f32),
    /// Quit application.
    Quit,
    /// Adjust sensor angle.
    AdjustSensorAngle(f32),
    /// Adjust sensor distance.
    AdjustSensorDistance(f32),
    /// Adjust turn angle.
    AdjustTurnAngle(f32),
    /// Adjust step size.
    AdjustStepSize(f32),
    /// Adjust decay factor.
    AdjustDecay(f32),
    /// Adjust deposit amount.
    AdjustDeposit(f32),
    /// Cycle diffusion kernel type.
    CycleDiffusionKernel,
    /// Adjust diffusion sigma.
    AdjustDiffusionSigma(f32),
    /// Adjust attractor strength.
    AdjustAttractorStrength(f32),
    /// Cycle mouse interaction mode.
    CycleMouseMode,
    /// Cycle wind direction.
    CycleWindDirection,
    /// Cycle wind direction in reverse.
    CycleWindDirectionReverse,
    /// Adjust terrain strength.
    AdjustTerrainStrength(f32),
    /// Cycle terrain type.
    CycleTerrainType,
    /// Toggle auto-normalization.
    ToggleAutoNormalize,
    /// Cycle motion blur amount.
    CycleMotionBlur,
    /// Adjust max brightness target.
    AdjustMaxBrightness(f32),
    /// Save current frame to PNG.
    SaveFrameToPng,
    /// Toggle fast rendering mode.
    ToggleFastMode,
    /// Cycle palette shift speed.
    CyclePaletteShiftSpeed,
    /// Toggle inverted palette.
    ToggleInvertPalette,
    /// Toggle reversed palette.
    ToggleReversePalette,
    /// Cycle to next intensity mapping.
    CycleIntensityMapping,
    /// Cycle to previous intensity mapping.
    CycleIntensityMappingReverse,
    /// Set specific intensity mapping.
    SetIntensityMapping(usize),
    /// Reset parameters to defaults.
    ResetToDefaults,
    /// Cycle controls category forward.
    CycleOptionsCategory,
    /// Cycle controls category backward.
    CycleOptionsCategoryReverse,
    /// Toggle dashboard overlay (merged stats + info).
    ToggleDashboard,
    /// Show configuration browser.
    ShowConfigBrowser,
    /// Show configuration save dialog.
    ShowConfigSaveDialog,
    /// Randomize all parameters.
    RandomizeParams,
    /// Undo last parameter change.
    Undo,
    /// Redo last undone change.
    Redo,
    /// Cycle to next UI theme.
    CycleTheme,
    /// Cycle to previous UI theme.
    CycleThemeReverse,
    /// Show palette editor.
    ShowPaletteEditor,
    /// Toggle trail age hue shifting.
    ToggleTrailAge,
    /// Toggle temporal delta brightness boost.
    ToggleTrailDelta,
    /// Toggle gradient magnitude edge glow.
    ToggleGradientMagnitude,
    /// Cycle to next window frame mode.
    CycleWindowFrame,
    /// Cycle to previous window frame mode.
    CycleWindowFrameReverse,
    /// No action.
    None,
}

#[derive(Debug, Clone, PartialEq)]
/// Snapshot of all simulation parameters for undo/redo.
pub struct ParameterState {
    /// Sensor angle.
    pub sensor_angle: f32,
    /// Sensor distance.
    pub sensor_distance: f32,
    /// Rotation angle (max turn per step).
    pub rotation_angle: f32,
    /// Step size.
    pub step_size: f32,
    /// Decay factor.
    pub decay_factor: f32,
    /// Deposit amount.
    pub deposit_amount: f32,
    /// Diffusion kernel.
    pub diffusion_kernel: DiffusionKernel,
    /// Diffusion sigma.
    pub diffusion_sigma: f32,
    /// Attractor strength.
    pub attractor_strength: f32,
    /// Wind direction.
    pub wind_direction: WindDirection,
    /// Terrain type.
    pub terrain_type: TerrainType,
    /// Terrain strength.
    pub terrain_strength: f32,
    /// Max brightness.
    pub max_brightness: f32,
    /// Palette index.
    pub palette_index: usize,
    /// Charset index.
    pub charset_index: usize,
    /// Invert palette flag.
    pub invert_palette: bool,
    /// Reverse palette flag.
    pub reverse_palette: bool,
    /// Dither mode.
    pub dither_mode: DitherMode,
    /// Motion blur frames.
    pub motion_blur_frames: usize,
    /// Window frame display mode.
    pub window_frame: WindowFrame,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// Default parameter values for reset functionality.
pub struct DefaultValues {
    /// Sensor angle.
    pub sensor_angle: f32,
    /// Sensor distance.
    pub sensor_distance: f32,
    /// Rotation angle (max turn per step).
    pub rotation_angle: f32,
    /// Step size.
    pub step_size: f32,
    /// Decay factor.
    pub decay_factor: f32,
    /// Deposit amount.
    pub deposit_amount: f32,
    /// Diffusion kernel.
    pub diffusion_kernel: DiffusionKernel,
    /// Diffusion sigma.
    pub diffusion_sigma: f32,
    /// Attractor strength.
    pub attractor_strength: f32,
    /// Wind direction.
    pub wind_direction: WindDirection,
    /// Terrain type.
    pub terrain_type: TerrainType,
    /// Terrain strength.
    pub terrain_strength: f32,
    /// Auto normalize enabled.
    pub auto_normalize: bool,
    /// Motion blur frames.
    pub motion_blur_frames: usize,
    /// Max brightness.
    pub max_brightness: f32,
}

impl DefaultValues {
    /// Create default values from a preset.
    pub fn from_preset(preset: Preset) -> Self {
        let config = SimConfig::from(preset);
        Self {
            sensor_angle: config.sensor_angle,
            sensor_distance: config.sensor_distance,
            rotation_angle: config.rotation_angle,
            step_size: config.step_size,
            decay_factor: config.decay_factor,
            deposit_amount: config.deposit_amount,
            diffusion_kernel: config.diffusion_kernel,
            diffusion_sigma: config.diffusion_sigma,
            attractor_strength: config.attractor_strength,
            wind_direction: match config.wind {
                None => WindDirection::None,
                Some(w) => {
                    if w.dx > 0.0 && w.dy == 0.0 {
                        WindDirection::East
                    } else if w.dx < 0.0 && w.dy == 0.0 {
                        WindDirection::West
                    } else if w.dx == 0.0 && w.dy < 0.0 {
                        WindDirection::North
                    } else if w.dx == 0.0 && w.dy > 0.0 {
                        WindDirection::South
                    } else {
                        WindDirection::None
                    }
                }
            },
            terrain_type: config.terrain,
            terrain_strength: config.terrain_strength,
            auto_normalize: false,
            motion_blur_frames: 0,
            max_brightness: config.max_brightness,
        }
    }
}

#[derive(Debug, Clone)]
/// Global runtime state managing simulation parameters and UI state.
pub struct RuntimeState {
    /// Whether simulation is paused.
    pub is_paused: bool,
    /// Flag set when pause is toggled, cleared after immediate re-render
    pub pause_just_toggled: bool,
    /// Centralized overlay state management.
    /// Replaces: show_controls, show_keyboard_hints, show_preset_comparison,
    /// show_dashboard, show_config_browser, show_config_save_dialog
    pub overlay_state: OverlayState,
    /// Preset being compared against.
    pub comparison_preset: Preset,
    /// Current category tab in controls overlay.
    pub controls_category_idx: usize,
    /// Time scale multiplier.
    pub time_scale: f32,
    /// Currently active preset.
    pub current_preset: Preset,
    /// Index of current palette.
    pub palette_index: usize,
    /// Index of current charset.
    pub charset_index: usize,
    /// Random seed used for initialization.
    pub original_seed: u64,
    /// Initialization mode used.
    pub original_init_mode: InitMode,
    /// Current dithering mode.
    pub dither_mode: DitherMode,
    /// Last used dither mode (for toggling).
    pub last_dither_mode: Option<DitherMode>,
    /// Current mouse interaction mode.
    pub mouse_mode: MouseInteractionMode,
    /// Timeout for mouse effects.
    pub mouse_timeout: f32,
    /// Sensor angle.
    pub sensor_angle: f32,
    /// Sensor distance.
    pub sensor_distance: f32,
    /// Rotation angle (max turn per step).
    pub rotation_angle: f32,
    /// Step size.
    pub step_size: f32,
    /// Decay factor.
    pub decay_factor: f32,
    /// Deposit amount.
    pub deposit_amount: f32,
    /// Diffusion kernel.
    pub diffusion_kernel: DiffusionKernel,
    /// Diffusion sigma.
    pub diffusion_sigma: f32,
    /// Attractor strength.
    pub attractor_strength: f32,
    /// Wind direction.
    pub wind_direction: WindDirection,
    /// Terrain type.
    pub terrain_type: TerrainType,
    /// Terrain strength.
    pub terrain_strength: f32,
    /// Auto normalize enabled.
    pub auto_normalize: bool,
    /// Motion blur frames.
    pub motion_blur_frames: usize,
    /// Window frame display mode.
    pub window_frame: WindowFrame,
    /// Chrome display style (minimal, expanded, fullscreen).
    pub chrome_style: ChromeStyle,
    /// Visual aspect ratio of the simulation window.
    pub aspect: Aspect,
    /// Outer padding between terminal edge and window frame.
    pub window_padding: WindowPadding,
    /// Show legacy status bar in windowed mode.
    pub show_status_bar: bool,
    /// Minimum sim size before dropping padding.
    pub min_sim_size: TerminalSizeThreshold,
    /// Minimum sim size before dropping the frame.
    pub min_frame_size: TerminalSizeThreshold,
    /// Max brightness.
    pub max_brightness: f32,
    /// Fast mode enabled.
    pub fast_mode_enabled: bool,
    /// Palette shift speed.
    pub palette_shift_speed: PaletteShiftSpeed,
    /// Invert palette.
    pub invert_palette: bool,
    /// Reverse palette.
    pub reverse_palette: bool,
    /// Intensity mapping for non-linear color distribution.
    pub intensity_mapping: IntensityMapping,
    /// Index of current intensity mapping preset.
    pub intensity_mapping_index: usize,
    /// Saved palette name (if loaded from saved palette).
    pub saved_palette_name: Option<String>,
    /// Current notification message with timestamp and severity level.
    pub notification: Option<(String, std::time::Instant, NotificationLevel)>,
    /// Frame counter for entropy collapse detection.
    pub collapse_frame_counter: usize,
    /// Warmup frame counter.
    pub warmup_counter: usize,
    /// Food persistence counter.
    pub food_persist_counter: usize,
    /// Food persistence enabled.
    pub food_persist_enabled: bool,
    /// Initial food attractors.
    pub initial_food_attractors: Vec<crate::simulation::config::Attractor>,
    /// Selected index in config browser.
    pub config_browser_selected_index: usize,
    /// Input buffer for save dialog.
    pub config_save_name_input: String,
    /// Default values for reset.
    pub default_values: DefaultValues,
    /// CLI overrides for custom parameters (stored when launched with CLI args).
    pub cli_overrides: Option<SimConfig>,
    /// Undo history stack.
    pub undo_stack: std::collections::VecDeque<ParameterState>,
    /// Redo history stack.
    pub redo_stack: std::collections::VecDeque<ParameterState>,
    /// Time of last undo checkpoint.
    pub last_checkpoint_time: std::time::Instant,
    /// Recent FPS history.
    pub fps_history: std::collections::VecDeque<f32>,
    /// Recent entropy history.
    pub entropy_history: std::collections::VecDeque<f32>,
    /// Recent density history.
    pub density_history: std::collections::VecDeque<f32>,
    /// Current panel theme/style.
    pub panel_style: PanelStyle,
    /// Index into `ALL_THEMES` for the active UI theme.
    pub theme_index: usize,
    /// Whether shift key is currently held down.
    pub shift_held: bool,
    /// Cached logo brightness map for VCR pause effect: (logo_w, pixel_h, data).
    /// Invalidated when `logo_w` changes (terminal resize).
    pub pause_logo_cache: Option<(usize, usize, Vec<f32>)>,
    /// Frame counter used for badge blink while paused.
    pub pause_frame_counter: u64,
    /// Pause screen visual effect style.
    pub pause_style: PauseStyle,
    /// Whether to show the logo image during pause state.
    pub pause_logo_enabled: bool,
    /// Debug mode: draw wave rings on empty cells in pulse pause effect.
    pub pause_pulse_draw_mode: bool,
    /// Trail age hue shifting enabled.
    pub trail_age_enabled: bool,
    /// Temporal delta brightness boost enabled.
    pub trail_delta_enabled: bool,
    /// Gradient magnitude edge glow enabled.
    pub gradient_magnitude_enabled: bool,
    /// Gradient magnitude strength for edge glow.
    pub gradient_strength: f32,
    /// Trail age hue shift range in degrees.
    pub trail_age_hue_range: f32,
    /// Trail age blend factor between original and age-modified colors.
    pub trail_age_blend: f32,
    /// Trail age hue shift mode (bidirectional or alternating).
    pub trail_age_mode: crate::config_defaults::TrailAgeMode,
    /// Reverse trail age bidirectional hue shift.
    pub trail_age_reverse: bool,
    /// Trail delta brightness boost strength.
    pub trail_delta_strength: f32,
}

impl RuntimeState {
    /// Creates a new runtime state with CLI-provided config values.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        seed: u64,
        init_mode: InitMode,
        initial_preset: Preset,
        initial_palette_index: usize,
        initial_charset_index: usize,
        mouse_mode: MouseInteractionMode,
        mouse_timeout: f32,
        intensity_mapping: IntensityMapping,
        cli_config: &SimConfig,
        pause_style: PauseStyle,
        pause_logo_enabled: bool,
        pause_pulse_draw_mode: bool,
    ) -> Self {
        let default_values = DefaultValues::from_preset(initial_preset);
        Self {
            is_paused: false,
            pause_just_toggled: false,
            overlay_state: OverlayState::default(),
            comparison_preset: initial_preset,
            controls_category_idx: 0,
            time_scale: cli_config.time_scale,
            current_preset: initial_preset,
            palette_index: initial_palette_index,
            charset_index: initial_charset_index,
            original_seed: seed,
            original_init_mode: init_mode,
            dither_mode: DitherMode::None,
            last_dither_mode: None,
            mouse_mode,
            mouse_timeout,
            sensor_angle: cli_config.sensor_angle,
            sensor_distance: cli_config.sensor_distance,
            rotation_angle: cli_config.rotation_angle,
            step_size: cli_config.step_size,
            decay_factor: cli_config.decay_factor,
            deposit_amount: cli_config.deposit_amount,
            diffusion_kernel: cli_config.diffusion_kernel,
            diffusion_sigma: cli_config.diffusion_sigma,
            attractor_strength: cli_config.attractor_strength,
            wind_direction: match cli_config.wind {
                None => WindDirection::None,
                Some(w) => {
                    if w.dx > 0.0 && w.dy == 0.0 {
                        WindDirection::East
                    } else if w.dx < 0.0 && w.dy == 0.0 {
                        WindDirection::West
                    } else if w.dx == 0.0 && w.dy < 0.0 {
                        WindDirection::North
                    } else if w.dx == 0.0 && w.dy > 0.0 {
                        WindDirection::South
                    } else {
                        WindDirection::None
                    }
                }
            },
            terrain_type: cli_config.terrain,
            terrain_strength: cli_config.terrain_strength,
            auto_normalize: false,
            motion_blur_frames: 0,
            window_frame: cli_config.window_frame,
            chrome_style: cli_config.chrome_style,
            aspect: cli_config.aspect,
            window_padding: cli_config.window_padding,
            show_status_bar: cli_config.show_status_bar,
            min_sim_size: cli_config.min_sim_size,
            min_frame_size: cli_config.min_frame_size,
            max_brightness: cli_config.max_brightness,
            fast_mode_enabled: false,
            palette_shift_speed: PaletteShiftSpeed::Off,
            invert_palette: false,
            reverse_palette: false,
            intensity_mapping: intensity_mapping.clone(),
            intensity_mapping_index: Self::find_intensity_mapping_index(&intensity_mapping),
            saved_palette_name: None,
            notification: None,
            collapse_frame_counter: 0,
            warmup_counter: 0,
            food_persist_counter: 0,
            food_persist_enabled: false,
            initial_food_attractors: Vec::new(),
            config_browser_selected_index: 0,
            config_save_name_input: String::new(),
            default_values,
            cli_overrides: Some(cli_config.clone()),
            undo_stack: std::collections::VecDeque::with_capacity(50),
            redo_stack: std::collections::VecDeque::with_capacity(50),
            last_checkpoint_time: std::time::Instant::now(),
            fps_history: std::collections::VecDeque::with_capacity(60),
            entropy_history: std::collections::VecDeque::with_capacity(60),
            density_history: std::collections::VecDeque::with_capacity(60),
            panel_style: GRUVBOX_DARK,
            theme_index: 0,
            shift_held: false,
            pause_logo_cache: None,
            pause_frame_counter: 0,
            pause_style,
            pause_logo_enabled,
            pause_pulse_draw_mode,
            trail_age_enabled: false,
            trail_delta_enabled: false,
            gradient_magnitude_enabled: false,
            gradient_strength: 0.3,
            trail_age_hue_range: 15.0,
            trail_age_blend: 0.5,
            trail_age_mode: crate::config_defaults::TrailAgeMode::Bidirectional,
            trail_age_reverse: true,
            trail_delta_strength: 0.5,
        }
    }

    /// Captures the current state of parameters for undo.
    pub fn capture_parameter_state(&self) -> ParameterState {
        ParameterState {
            sensor_angle: self.sensor_angle,
            sensor_distance: self.sensor_distance,
            rotation_angle: self.rotation_angle,
            step_size: self.step_size,
            decay_factor: self.decay_factor,
            deposit_amount: self.deposit_amount,
            diffusion_kernel: self.diffusion_kernel,
            diffusion_sigma: self.diffusion_sigma,
            attractor_strength: self.attractor_strength,
            wind_direction: self.wind_direction,
            terrain_type: self.terrain_type,
            terrain_strength: self.terrain_strength,
            max_brightness: self.max_brightness,
            palette_index: self.palette_index,
            charset_index: self.charset_index,
            invert_palette: self.invert_palette,
            reverse_palette: self.reverse_palette,
            dither_mode: self.dither_mode,
            motion_blur_frames: self.motion_blur_frames,
            window_frame: self.window_frame,
        }
    }

    /// Restores parameters from a saved state.
    pub fn apply_parameter_state(&mut self, state: ParameterState) {
        self.sensor_angle = state.sensor_angle;
        self.sensor_distance = state.sensor_distance;
        self.rotation_angle = state.rotation_angle;
        self.step_size = state.step_size;
        self.decay_factor = state.decay_factor;
        self.deposit_amount = state.deposit_amount;
        self.diffusion_kernel = state.diffusion_kernel;
        self.diffusion_sigma = state.diffusion_sigma;
        self.attractor_strength = state.attractor_strength;
        self.wind_direction = state.wind_direction;
        self.terrain_type = state.terrain_type;
        self.terrain_strength = state.terrain_strength;
        self.max_brightness = state.max_brightness;
        self.palette_index = state.palette_index;
        self.charset_index = state.charset_index;
        self.invert_palette = state.invert_palette;
        self.reverse_palette = state.reverse_palette;
        self.dither_mode = state.dither_mode;
        self.motion_blur_frames = state.motion_blur_frames;
        self.window_frame = state.window_frame;
    }

    /// Creates an undo checkpoint if enough time has passed.
    pub fn checkpoint(&mut self) {
        if self.last_checkpoint_time.elapsed().as_millis() < 500 {
            return;
        }

        let current = self.capture_parameter_state();
        if let Some(last) = self.undo_stack.back() {
            if last == &current {
                return;
            }
        }

        self.undo_stack.push_back(current);
        if self.undo_stack.len() > 50 {
            self.undo_stack.pop_front();
        }
        self.redo_stack.clear();
        self.last_checkpoint_time = std::time::Instant::now();
    }

    /// Forces creation of an undo checkpoint regardless of time.
    pub fn force_checkpoint(&mut self) {
        let current = self.capture_parameter_state();
        self.undo_stack.push_back(current);
        if self.undo_stack.len() > 50 {
            self.undo_stack.pop_front();
        }
        self.redo_stack.clear();
        self.last_checkpoint_time = std::time::Instant::now();
    }

    /// Undoes the last parameter change.
    pub fn undo(&mut self) -> Option<ParameterState> {
        if self.undo_stack.is_empty() {
            return None;
        }

        let current = self.capture_parameter_state();
        self.redo_stack.push_back(current);

        let previous = self.undo_stack.pop_back().unwrap();
        self.apply_parameter_state(previous.clone());
        Some(previous)
    }

    /// Redoes the last undone change.
    pub fn redo(&mut self) -> Option<ParameterState> {
        if self.redo_stack.is_empty() {
            return None;
        }

        let current = self.capture_parameter_state();
        self.undo_stack.push_back(current);

        let next = self.redo_stack.pop_back().unwrap();
        self.apply_parameter_state(next.clone());
        Some(next)
    }

    /// Toggles the paused state.
    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    /// Pre-loads the pause logo to avoid delay on first pause.
    ///
    /// Calculates logo size from terminal dimensions and loads the embedded
    /// PNG using Lanczos3 filtering, then caches the result.
    pub fn preload_pause_logo(&mut self, term_width: usize, _term_height: usize) {
        // Use same percentage logic as pause rendering to ensure cache hits
        let pct = if term_width < 80 {
            0.90
        } else if term_width < 120 {
            0.75
        } else {
            0.60
        };
        let logo_w = ((term_width as f32 * pct) as usize).clamp(30, 180);
        let logo_h = ((logo_w as f32 / 2.67) as usize).max(6);
        let pixel_w = logo_w * 2;
        let pixel_h = logo_h * 2;

        let map = load_logo_from_memory(FOOD_IMAGE_PNG, pixel_w, pixel_h, true)
            .unwrap_or_else(|_| vec![0.0; pixel_w * pixel_h]);
        self.pause_logo_cache = Some((logo_w, pixel_h, map));
    }

    /// Toggles the controls overlay.
    pub fn toggle_controls(&mut self) {
        self.overlay_state.toggle(OverlayType::Controls);
    }

    /// Toggles the keyboard shortcuts overlay.
    pub fn toggle_keyboard_hints(&mut self) {
        self.overlay_state.toggle(OverlayType::KeyboardHints);
    }

    /// Toggles the preset comparison overlay.
    pub fn toggle_preset_comparison(&mut self, preset: Preset) {
        if self.overlay_state.is_open(OverlayType::PresetComparison)
            && self.comparison_preset == preset
        {
            self.overlay_state.close();
        } else {
            self.overlay_state.open(OverlayType::PresetComparison);
            self.comparison_preset = preset;
        }
    }

    /// Checks if any overlay window is currently open.
    pub fn any_overlay_open(&self) -> bool {
        self.overlay_state.any_open()
    }

    /// Closes all open overlay windows.
    pub fn close_all_overlays(&mut self) {
        self.overlay_state.close();
    }

    /// Toggles palette editor (mutually exclusive with other overlays).
    pub fn toggle_palette_editor(&mut self) {
        self.overlay_state.toggle(OverlayType::PaletteEditor);
    }

    /// Returns whether the given overlay type is currently active.
    pub fn is_overlay_active(&self, overlay: OverlayType) -> bool {
        self.overlay_state.is_open(overlay)
    }

    /// Cycles through control categories.
    pub fn cycle_controls_category(&mut self, forward: bool) {
        if forward {
            self.controls_category_idx =
                (self.controls_category_idx + 1) % ControlsOverlay::TOTAL_CATEGORIES;
        } else {
            self.controls_category_idx = if self.controls_category_idx == 0 {
                ControlsOverlay::TOTAL_CATEGORIES - 1
            } else {
                self.controls_category_idx - 1
            };
        }
    }

    /// Applies a preset configuration.
    pub fn set_preset(&mut self, preset: Preset) {
        self.force_checkpoint();
        self.current_preset = preset;
        self.default_values = DefaultValues::from_preset(preset);
    }

    /// Adjusts simulation time scale.
    pub fn adjust_time_scale(&mut self, delta: f32) {
        self.checkpoint();
        let new_scale = (self.time_scale + delta).clamp(0.5, 10.0);
        self.time_scale = new_scale;
    }

    /// Available intensity mapping presets.
    #[allow(clippy::type_complexity)]
    pub const INTENSITY_MAPPINGS: &'static [(&'static str, fn() -> IntensityMapping)] = &[
        ("Linear", || IntensityMapping::linear()),
        ("Logarithmic", || IntensityMapping::logarithmic(10.0)),
        ("Exponential", || IntensityMapping::exponential(10.0)),
        ("Square Root", || {
            IntensityMapping::new(vec![crate::render::palette::MappingSegment {
                start: 0.0,
                end: 1.0,
                function: crate::render::palette::MappingFunction::SquareRoot,
            }])
            .unwrap()
        }),
        ("Smoothstep", || IntensityMapping::smoothstep()),
        ("Split (Lin/Log)", || {
            IntensityMapping::linear_log_split(10.0)
        }),
        ("Quantize 6", || IntensityMapping::quantize(6)),
        ("Perlin", || {
            IntensityMapping::perlin(0.15, 4.0, PERLIN_SEED)
        }),
    ];

    /// Finds the index of a given intensity mapping by comparing with presets.
    /// Returns 0 if no match found (falls back to Linear).
    fn find_intensity_mapping_index(mapping: &IntensityMapping) -> usize {
        for (i, (_, factory)) in Self::INTENSITY_MAPPINGS.iter().enumerate() {
            if &factory() == mapping {
                return i;
            }
        }
        0
    }

    /// Cycles through the available intensity mapping presets.
    pub fn cycle_intensity_mapping(&mut self, reverse: bool) {
        let count = Self::INTENSITY_MAPPINGS.len();
        if reverse {
            self.intensity_mapping_index = (self.intensity_mapping_index + count - 1) % count;
        } else {
            self.intensity_mapping_index = (self.intensity_mapping_index + 1) % count;
        }
        let (_, factory) = Self::INTENSITY_MAPPINGS[self.intensity_mapping_index];
        self.intensity_mapping = factory();
    }

    /// Returns the name of the current intensity mapping preset.
    pub fn intensity_mapping_name(&self) -> &'static str {
        Self::INTENSITY_MAPPINGS[self.intensity_mapping_index].0
    }

    /// Cycles to the next color palette.
    pub fn cycle_palette(&mut self, num_palettes: usize) {
        self.force_checkpoint();
        self.palette_index = (self.palette_index + 1) % num_palettes;
    }

    /// Cycles to the previous color palette.
    pub fn cycle_palette_reverse(&mut self, num_palettes: usize) {
        self.force_checkpoint();
        if self.palette_index == 0 {
            self.palette_index = num_palettes - 1;
        } else {
            self.palette_index -= 1;
        }
    }

    /// Cycles to the next charset.
    pub fn cycle_charset(&mut self) {
        self.force_checkpoint();
        self.charset_index = (self.charset_index + 1) % ALL_CHARSETS.len();
    }

    /// Cycles to the previous charset.
    pub fn cycle_charset_reverse(&mut self) {
        self.force_checkpoint();
        if self.charset_index == 0 {
            self.charset_index = ALL_CHARSETS.len() - 1;
        } else {
            self.charset_index -= 1;
        }
    }

    /// Cycles to the next window frame mode.
    pub fn cycle_window_frame(&mut self) {
        self.force_checkpoint();
        self.window_frame = match self.window_frame {
            WindowFrame::None => WindowFrame::Negative,
            WindowFrame::Negative => WindowFrame::Accented,
            WindowFrame::Accented => WindowFrame::Glow,
            WindowFrame::Glow => WindowFrame::Reactive,
            WindowFrame::Reactive => WindowFrame::Food,
            WindowFrame::Food => WindowFrame::Frame,
            WindowFrame::Frame => WindowFrame::None,
        };
    }

    /// Cycles to the previous window frame mode.
    pub fn cycle_window_frame_reverse(&mut self) {
        self.force_checkpoint();
        self.window_frame = match self.window_frame {
            WindowFrame::None => WindowFrame::Frame,
            WindowFrame::Negative => WindowFrame::None,
            WindowFrame::Accented => WindowFrame::Negative,
            WindowFrame::Glow => WindowFrame::Accented,
            WindowFrame::Reactive => WindowFrame::Glow,
            WindowFrame::Food => WindowFrame::Reactive,
            WindowFrame::Frame => WindowFrame::Food,
        };
    }

    /// Gets the currently active charset.
    pub fn current_charset(&self) -> Charset {
        ALL_CHARSETS[self.charset_index].clone()
    }

    /// Gets the currently active palette.
    pub fn current_palette(&self, palettes: &[Palette; NUM_PALETTES]) -> Palette {
        palettes[self.palette_index].clone()
    }

    /// Toggles dithering on/off.
    pub fn toggle_dither(&mut self) {
        self.force_checkpoint();
        self.dither_mode = match self.dither_mode {
            DitherMode::None => {
                if let Some(last) = self.last_dither_mode {
                    last
                } else {
                    DitherMode::Ordered {
                        intensity: dither_consts::DEFAULT_INTENSITY,
                        matrix: DitherMatrix::Bayer4x4,
                    }
                }
            }
            mode => {
                self.last_dither_mode = Some(mode);
                DitherMode::None
            }
        };
    }

    /// Cycles through available dithering modes.
    pub fn cycle_dither_mode(&mut self) {
        self.force_checkpoint();
        self.dither_mode = match self.dither_mode {
            DitherMode::None => DitherMode::Ordered {
                intensity: dither_consts::DEFAULT_INTENSITY,
                matrix: DitherMatrix::Bayer4x4,
            },
            DitherMode::Ordered {
                intensity: _,
                matrix: _,
            } => DitherMode::ErrorDiffusion { serpentine: true },
            DitherMode::ErrorDiffusion { .. } => DitherMode::Hybrid {
                edge_threshold: dither_consts::DEFAULT_HYBRID_EDGE_THRESHOLD,
                intensity: dither_consts::DEFAULT_INTENSITY,
                matrix: DitherMatrix::Bayer4x4,
            },
            DitherMode::Hybrid { .. } => DitherMode::None,
        };
        if self.dither_mode != DitherMode::None {
            self.last_dither_mode = Some(self.dither_mode);
        }
    }

    /// Adjusts dithering intensity.
    pub fn adjust_dither_intensity(&mut self, delta: f32) {
        self.checkpoint();
        self.dither_mode = match self.dither_mode {
            DitherMode::Ordered { intensity, matrix } => {
                let new_intensity = (intensity + delta).clamp(0.0, 1.0);
                DitherMode::Ordered {
                    intensity: new_intensity,
                    matrix,
                }
            }
            DitherMode::Hybrid {
                edge_threshold,
                intensity,
                matrix,
            } => {
                let new_intensity = (intensity + delta).clamp(0.0, 1.0);
                DitherMode::Hybrid {
                    edge_threshold,
                    intensity: new_intensity,
                    matrix,
                }
            }
            _ => self.dither_mode,
        };
    }

    /// Adjusts sensor angle.
    pub fn adjust_sensor_angle(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.sensor_angle + delta).clamp(
            agent_consts::MIN_SENSOR_ANGLE,
            agent_consts::MAX_SENSOR_ANGLE,
        );
        let at_bound = (new_value - self.sensor_angle).abs() < 0.01;
        self.sensor_angle = new_value;
        at_bound
    }

    /// Adjusts sensor distance.
    pub fn adjust_sensor_distance(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.sensor_distance + delta).clamp(
            agent_consts::MIN_SENSOR_DISTANCE,
            agent_consts::MAX_SENSOR_DISTANCE,
        );
        let at_bound = (new_value - self.sensor_distance).abs() < 0.01;
        self.sensor_distance = new_value;
        at_bound
    }

    /// Adjusts rotation angle.
    pub fn adjust_rotation_angle(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.rotation_angle + delta).clamp(
            agent_consts::MIN_ROTATION_ANGLE,
            agent_consts::MAX_ROTATION_ANGLE,
        );
        let at_bound = (new_value - self.rotation_angle).abs() < 0.01;
        self.rotation_angle = new_value;
        at_bound
    }

    /// Adjusts step size.
    pub fn adjust_step_size(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.step_size + delta)
            .clamp(agent_consts::MIN_STEP_SIZE, agent_consts::MAX_STEP_SIZE);
        let at_bound = (new_value - self.step_size).abs() < 0.01;
        self.step_size = new_value;
        at_bound
    }

    /// Adjusts decay factor.
    pub fn adjust_decay(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value =
            (self.decay_factor + delta).clamp(trail::MIN_DECAY_FACTOR, trail::MAX_DECAY_FACTOR);
        let at_bound = (new_value - self.decay_factor).abs() < 0.001;
        self.decay_factor = new_value;
        at_bound
    }

    /// Adjusts deposit amount.
    pub fn adjust_deposit(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.deposit_amount + delta).clamp(
            agent_consts::MIN_DEPOSIT_AMOUNT,
            agent_consts::MAX_DEPOSIT_AMOUNT,
        );
        let at_bound = (new_value - self.deposit_amount).abs() < 0.01;
        self.deposit_amount = new_value;
        at_bound
    }

    /// Cycles through diffusion kernels.
    pub fn cycle_diffusion_kernel(&mut self) {
        self.force_checkpoint();
        self.diffusion_kernel = match self.diffusion_kernel {
            DiffusionKernel::Mean3x3 => DiffusionKernel::Gaussian,
            DiffusionKernel::Gaussian => DiffusionKernel::Mean3x3,
        };
    }

    /// Adjusts diffusion sigma (for Gaussian kernel).
    pub fn adjust_diffusion_sigma(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.diffusion_sigma + delta)
            .clamp(trail::MIN_DIFFUSION_SIGMA, trail::MAX_DIFFUSION_SIGMA);
        let at_bound = (new_value - self.diffusion_sigma).abs() < 0.01;
        self.diffusion_sigma = new_value;
        at_bound
    }

    /// Adjusts global attractor strength.
    pub fn adjust_attractor_strength(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.attractor_strength + delta).clamp(
            env_consts::MIN_ATTRACTOR_STRENGTH,
            env_consts::MAX_ATTRACTOR_STRENGTH,
        );
        let at_bound = (new_value - self.attractor_strength).abs() < 0.01;
        self.attractor_strength = new_value;
        at_bound
    }

    /// Cycles mouse interaction mode.
    pub fn cycle_mouse_mode(&mut self) {
        self.force_checkpoint();
        self.mouse_mode = match self.mouse_mode {
            MouseInteractionMode::Disabled => MouseInteractionMode::Attract,
            MouseInteractionMode::Attract => MouseInteractionMode::Repel,
            MouseInteractionMode::Repel => MouseInteractionMode::Disabled,
        };
    }

    /// Cycles wind direction.
    pub fn cycle_wind_direction(&mut self) {
        self.force_checkpoint();
        self.wind_direction = match self.wind_direction {
            WindDirection::None => WindDirection::North,
            WindDirection::North => WindDirection::Northeast,
            WindDirection::Northeast => WindDirection::East,
            WindDirection::East => WindDirection::Southeast,
            WindDirection::Southeast => WindDirection::South,
            WindDirection::South => WindDirection::Southwest,
            WindDirection::Southwest => WindDirection::West,
            WindDirection::West => WindDirection::Northwest,
            WindDirection::Northwest => WindDirection::None,
        };
    }

    /// Cycles wind direction in reverse.
    pub fn cycle_wind_direction_reverse(&mut self) {
        self.force_checkpoint();
        self.wind_direction = match self.wind_direction {
            WindDirection::None => WindDirection::Northwest,
            WindDirection::North => WindDirection::None,
            WindDirection::Northeast => WindDirection::North,
            WindDirection::East => WindDirection::Northeast,
            WindDirection::Southeast => WindDirection::East,
            WindDirection::South => WindDirection::Southeast,
            WindDirection::Southwest => WindDirection::South,
            WindDirection::West => WindDirection::Southwest,
            WindDirection::Northwest => WindDirection::West,
        };
    }

    /// Adjusts terrain strength.
    pub fn adjust_terrain_strength(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.terrain_strength + delta).clamp(
            env_consts::MIN_TERRAIN_STRENGTH,
            env_consts::MAX_TERRAIN_STRENGTH,
        );
        let at_bound = (new_value - self.terrain_strength).abs() < 0.01;
        self.terrain_strength = new_value;
        at_bound
    }

    /// Cycles terrain type.
    pub fn cycle_terrain_type(&mut self) {
        self.force_checkpoint();
        self.terrain_type = match self.terrain_type {
            TerrainType::None => TerrainType::Smooth,
            TerrainType::Smooth => TerrainType::Turbulent,
            TerrainType::Turbulent => TerrainType::Mixed,
            TerrainType::Mixed => TerrainType::None,
        };
    }

    /// Toggles auto-normalization of brightness.
    pub fn toggle_auto_normalize(&mut self) {
        self.force_checkpoint();
        self.auto_normalize = !self.auto_normalize;
    }

    /// Cycles motion blur amount.
    pub fn cycle_motion_blur(&mut self) {
        self.force_checkpoint();
        self.motion_blur_frames = match self.motion_blur_frames {
            0 => 3,
            3 => 5,
            5 => 7,
            7 => 0,
            _ => 0,
        };
    }

    /// Adjusts max brightness target.
    pub fn adjust_max_brightness(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.max_brightness + delta)
            .clamp(trail::MIN_MAX_BRIGHTNESS, trail::MAX_MAX_BRIGHTNESS);
        let at_bound = (new_value - self.max_brightness).abs() < 0.01;
        self.max_brightness = new_value;
        at_bound
    }

    /// Toggles fast rendering mode.
    pub fn toggle_fast_mode(&mut self) {
        self.force_checkpoint();
        self.fast_mode_enabled = !self.fast_mode_enabled;
    }

    /// Cycles palette shift speed.
    pub fn cycle_palette_shift_speed(&mut self) {
        self.force_checkpoint();
        self.palette_shift_speed = match self.palette_shift_speed {
            PaletteShiftSpeed::Off => PaletteShiftSpeed::Slow,
            PaletteShiftSpeed::Slow => PaletteShiftSpeed::Medium,
            PaletteShiftSpeed::Medium => PaletteShiftSpeed::Fast,
            PaletteShiftSpeed::Fast => PaletteShiftSpeed::Off,
        };
    }

    /// Toggles inverted palette.
    pub fn toggle_invert_palette(&mut self) {
        self.force_checkpoint();
        self.invert_palette = !self.invert_palette;
    }

    /// Toggles reversed palette.
    pub fn toggle_reverse_palette(&mut self) {
        self.force_checkpoint();
        self.reverse_palette = !self.reverse_palette;
    }

    /// Toggles dashboard overlay (merged stats + info).
    pub fn toggle_dashboard(&mut self) {
        self.overlay_state.toggle(OverlayType::Dashboard);
    }

    /// Resets all parameters to default values.
    /// If CLI overrides are available, restores those; otherwise uses preset defaults.
    pub fn reset_to_defaults(&mut self) {
        self.force_checkpoint();

        if let Some(ref cli) = self.cli_overrides {
            self.sensor_angle = cli.sensor_angle;
            self.sensor_distance = cli.sensor_distance;
            self.rotation_angle = cli.rotation_angle;
            self.step_size = cli.step_size;
            self.decay_factor = cli.decay_factor;
            self.deposit_amount = cli.deposit_amount;
            self.diffusion_kernel = cli.diffusion_kernel;
            self.diffusion_sigma = cli.diffusion_sigma;
            self.attractor_strength = cli.attractor_strength;
            self.wind_direction = match cli.wind {
                None => WindDirection::None,
                Some(w) => {
                    if w.dx > 0.0 && w.dy == 0.0 {
                        WindDirection::East
                    } else if w.dx < 0.0 && w.dy == 0.0 {
                        WindDirection::West
                    } else if w.dx == 0.0 && w.dy < 0.0 {
                        WindDirection::North
                    } else if w.dx == 0.0 && w.dy > 0.0 {
                        WindDirection::South
                    } else {
                        WindDirection::None
                    }
                }
            };
            self.terrain_type = cli.terrain;
            self.terrain_strength = cli.terrain_strength;
            self.max_brightness = cli.max_brightness;
            self.time_scale = cli.time_scale;
        } else {
            let defaults = self.default_values;
            self.sensor_angle = defaults.sensor_angle;
            self.sensor_distance = defaults.sensor_distance;
            self.rotation_angle = defaults.rotation_angle;
            self.step_size = defaults.step_size;
            self.decay_factor = defaults.decay_factor;
            self.deposit_amount = defaults.deposit_amount;
            self.diffusion_kernel = defaults.diffusion_kernel;
            self.diffusion_sigma = defaults.diffusion_sigma;
            self.attractor_strength = defaults.attractor_strength;
            self.wind_direction = defaults.wind_direction;
            self.terrain_type = defaults.terrain_type;
            self.terrain_strength = defaults.terrain_strength;
            self.max_brightness = defaults.max_brightness;
        }
        self.auto_normalize = false;
        self.motion_blur_frames = 0;
        self.fast_mode_enabled = false;
        self.palette_shift_speed = PaletteShiftSpeed::Off;
        self.invert_palette = false;
        self.reverse_palette = false;
    }

    /// Randomizes simulation parameters.
    pub fn randomize_params(&mut self) {
        self.force_checkpoint();
        let mut rng = rand::thread_rng();

        self.sensor_angle = rng.gen_range(15.0..60.0);
        self.rotation_angle = rng.gen_range(15.0..60.0);
        self.step_size = rng.gen_range(0.5..2.5);
        self.decay_factor = rng.gen_range(0.80..0.98);
        self.deposit_amount = rng.gen_range(2.0..10.0);

        self.diffusion_kernel = if rng.gen_bool(0.3) {
            DiffusionKernel::Gaussian
        } else {
            DiffusionKernel::Mean3x3
        };

        self.terrain_type = match rng.gen_range(0..4) {
            0 => TerrainType::None,
            1 => TerrainType::Smooth,
            2 => TerrainType::Turbulent,
            _ => TerrainType::Mixed,
        };
        self.terrain_strength = rng.gen_range(0.5..3.0);

        self.palette_index = rng.gen_range(0..ALL_PALETTES.len());
        self.max_brightness = rng.gen_range(10.0..40.0);
    }

    /// Shows a temporary notification message at Info level.
    pub fn show_notification(&mut self, message: String) {
        self.notification = Some((message, std::time::Instant::now(), NotificationLevel::Info));
    }

    /// Shows a temporary notification with an explicit severity level.
    pub fn show_notification_with_level(&mut self, message: String, level: NotificationLevel) {
        self.notification = Some((message, std::time::Instant::now(), level));
    }

    /// Updates notification state (clears expired notifications).
    pub fn update_notifications(&mut self) {
        if let Some((_, time, _)) = self.notification {
            if time.elapsed().as_secs() >= 3 {
                self.notification = None;
            }
        }
    }

    /// Returns the current notification message if any.
    pub fn current_notification(&self) -> Option<&String> {
        self.notification.as_ref().map(|(msg, _, _)| msg)
    }

    /// Returns the current notification message and level if any.
    pub fn current_notification_full(&self) -> Option<(&str, NotificationLevel)> {
        self.notification
            .as_ref()
            .map(|(msg, _, level)| (msg.as_str(), *level))
    }

    /// Cycles to the next UI theme.
    pub fn cycle_theme(&mut self) {
        self.theme_index = (self.theme_index + 1) % ALL_THEMES.len();
        self.panel_style = ALL_THEMES[self.theme_index].style();
    }

    /// Cycles to the previous UI theme.
    pub fn cycle_theme_reverse(&mut self) {
        self.theme_index = self
            .theme_index
            .checked_sub(1)
            .unwrap_or(ALL_THEMES.len() - 1);
        self.panel_style = ALL_THEMES[self.theme_index].style();
    }

    /// Returns the display name of the current UI theme.
    pub fn current_theme_name(&self) -> &'static str {
        ALL_THEMES[self.theme_index].name()
    }

    /// Checks if the simulation is in the warmup phase.
    pub fn is_in_warmup(&self, warmup_frames: usize) -> bool {
        warmup_frames > 0 && self.warmup_counter < warmup_frames
    }

    /// Increments the warmup frame counter.
    pub fn increment_warmup(&mut self) {
        self.warmup_counter += 1;
    }

    /// Resets the warmup frame counter.
    pub fn reset_warmup(&mut self) {
        self.warmup_counter = 0;
    }

    /// Tracks entropy for collapse detection.
    ///
    /// Returns true if collapse detected (entropy > threshold for duration).
    pub fn track_entropy(&mut self, entropy: f32, threshold: f32, duration_frames: usize) -> bool {
        if entropy > threshold {
            self.collapse_frame_counter += 1;
            self.collapse_frame_counter >= duration_frames
        } else {
            self.collapse_frame_counter = 0;
            false
        }
    }

    /// Resets the collapse frame counter.
    pub fn reset_collapse_counter(&mut self) {
        self.collapse_frame_counter = 0;
    }

    /// Updates statistics history buffers.
    pub fn update_history(&mut self, fps: f32, entropy: f32, density: f32) {
        self.fps_history.push_back(fps);
        if self.fps_history.len() > 60 {
            self.fps_history.pop_front();
        }

        self.entropy_history.push_back(entropy);
        if self.entropy_history.len() > 60 {
            self.entropy_history.pop_front();
        }

        self.density_history.push_back(density);
        if self.density_history.len() > 60 {
            self.density_history.pop_front();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::PauseStyle;
    use crate::simulation::config::SimConfig;

    fn create_test_runtime_state() -> RuntimeState {
        RuntimeState::new(
            42,
            InitMode::Random,
            Preset::Network,
            0,
            0,
            MouseInteractionMode::Disabled,
            0.0,
            IntensityMapping::linear(),
            &SimConfig::default(),
            PauseStyle::Vignette,
            false,
            false,
        )
    }

    #[test]
    fn test_palette_shift_speed_cycling() {
        let mut state = create_test_runtime_state();

        assert_eq!(state.palette_shift_speed, PaletteShiftSpeed::Off);

        state.cycle_palette_shift_speed();
        assert_eq!(state.palette_shift_speed, PaletteShiftSpeed::Slow);

        state.cycle_palette_shift_speed();
        assert_eq!(state.palette_shift_speed, PaletteShiftSpeed::Medium);

        state.cycle_palette_shift_speed();
        assert_eq!(state.palette_shift_speed, PaletteShiftSpeed::Fast);

        state.cycle_palette_shift_speed();
        assert_eq!(state.palette_shift_speed, PaletteShiftSpeed::Off);
    }

    #[test]
    fn test_time_scale_adjustment() {
        let mut state = create_test_runtime_state();

        assert_eq!(state.time_scale, 1.0);

        state.adjust_time_scale(0.5);
        assert_eq!(state.time_scale, 1.5);

        state.adjust_time_scale(-0.5);
        assert_eq!(state.time_scale, 1.0);
    }

    #[test]
    fn test_controls_toggle() {
        let mut state = create_test_runtime_state();

        assert!(!state.overlay_state.is_open(OverlayType::Controls));

        state.toggle_controls();
        assert!(state.overlay_state.is_open(OverlayType::Controls));

        state.toggle_controls();
        assert!(!state.overlay_state.is_open(OverlayType::Controls));
    }

    #[test]
    fn test_any_overlay_open() {
        let mut state = create_test_runtime_state();

        assert!(!state.any_overlay_open());

        state.overlay_state.open(OverlayType::Controls);
        assert!(state.any_overlay_open());

        state.overlay_state.close();
        state.overlay_state.open(OverlayType::Dashboard);
        assert!(state.any_overlay_open());
    }

    #[test]
    fn test_close_all_overlays() {
        let mut state = create_test_runtime_state();

        state.overlay_state.open(OverlayType::Controls);
        state.overlay_state.open(OverlayType::Dashboard);

        state.close_all_overlays();

        assert!(!state.overlay_state.is_open(OverlayType::Controls));
        assert!(!state.overlay_state.is_open(OverlayType::Dashboard));
        assert!(!state.any_overlay_open());
    }

    #[test]
    fn test_controls_category_cycling() {
        let mut state = create_test_runtime_state();

        assert_eq!(state.controls_category_idx, 0);

        state.cycle_controls_category(true);
        assert_eq!(state.controls_category_idx, 1);

        state.cycle_controls_category(true);
        assert_eq!(state.controls_category_idx, 2);

        state.cycle_controls_category(true);
        assert_eq!(state.controls_category_idx, 3);

        state.cycle_controls_category(true);
        assert_eq!(state.controls_category_idx, 4);

        state.cycle_controls_category(true);
        assert_eq!(state.controls_category_idx, 5);

        state.cycle_controls_category(true);
        assert_eq!(state.controls_category_idx, 0);

        state.cycle_controls_category(false);
        assert_eq!(state.controls_category_idx, 5);
    }

    #[test]
    fn test_wind_direction_cycling() {
        let mut state = create_test_runtime_state();

        assert_eq!(state.wind_direction, WindDirection::None);

        state.cycle_wind_direction();
        assert_eq!(state.wind_direction, WindDirection::North);

        state.cycle_wind_direction();
        assert_eq!(state.wind_direction, WindDirection::Northeast);

        state.cycle_wind_direction();
        assert_eq!(state.wind_direction, WindDirection::East);

        state.cycle_wind_direction();
        assert_eq!(state.wind_direction, WindDirection::Southeast);

        state.cycle_wind_direction();
        assert_eq!(state.wind_direction, WindDirection::South);

        state.cycle_wind_direction();
        assert_eq!(state.wind_direction, WindDirection::Southwest);

        state.cycle_wind_direction();
        assert_eq!(state.wind_direction, WindDirection::West);

        state.cycle_wind_direction();
        assert_eq!(state.wind_direction, WindDirection::Northwest);

        state.cycle_wind_direction();
        assert_eq!(state.wind_direction, WindDirection::None);
    }

    #[test]
    fn test_wind_direction_names() {
        assert_eq!(WindDirection::None.name(), "None");
        assert_eq!(WindDirection::North.name(), "N");
        assert_eq!(WindDirection::Southwest.name(), "SW");
    }

    #[test]
    fn test_runtime_state_randomize() {
        let mut state = RuntimeState::new(
            42,
            InitMode::Random,
            Preset::Organic,
            0,
            0,
            MouseInteractionMode::Disabled,
            3.0,
            IntensityMapping::linear(),
            &SimConfig::default(),
            PauseStyle::Vignette,
            false,
            false,
        );
        let orig_angle = state.sensor_angle;
        state.randomize_params();
        // Since it's random, it *could* be the same, but very unlikely
        assert!(state.sensor_angle != orig_angle || state.rotation_angle != 45.0);
    }

    #[test]
    fn test_parameter_state_roundtrip() {
        let mut state = RuntimeState::new(
            42,
            InitMode::Random,
            Preset::Organic,
            0,
            0,
            MouseInteractionMode::Disabled,
            3.0,
            IntensityMapping::linear(),
            &SimConfig::default(),
            PauseStyle::Vignette,
            false,
            false,
        );
        state.sensor_angle = 12.3;
        let p = state.capture_parameter_state();
        state.sensor_angle = 45.6;
        state.apply_parameter_state(p);
        assert_eq!(state.sensor_angle, 12.3);
    }

    #[test]
    fn test_palette_shift_speed() {
        assert_eq!(PaletteShiftSpeed::Off.degrees_per_second(), 0.0);
        assert_eq!(PaletteShiftSpeed::Fast.degrees_per_second(), 45.0);
    }

    #[test]
    fn test_runtime_state_notifications() {
        let mut state = RuntimeState::new(
            42,
            InitMode::Random,
            Preset::Organic,
            0,
            0,
            MouseInteractionMode::Disabled,
            3.0,
            IntensityMapping::linear(),
            &SimConfig::default(),
            PauseStyle::Vignette,
            false,
            false,
        );
        assert_eq!(state.current_notification(), None);
        state.show_notification("test".to_string());
        assert_eq!(state.current_notification(), Some(&"test".to_string()));
        state.update_notifications();
        assert_eq!(state.current_notification(), Some(&"test".to_string()));
    }

    #[test]
    fn test_runtime_state_warmup() {
        let mut state = RuntimeState::new(
            42,
            InitMode::Random,
            Preset::Organic,
            0,
            0,
            MouseInteractionMode::Disabled,
            3.0,
            IntensityMapping::linear(),
            &SimConfig::default(),
            PauseStyle::Vignette,
            false,
            false,
        );
        assert!(!state.is_in_warmup(0));
        assert!(state.is_in_warmup(10));
        state.increment_warmup();
        assert_eq!(state.warmup_counter, 1);
        state.reset_warmup();
        assert_eq!(state.warmup_counter, 0);
    }

    #[test]
    fn test_runtime_state_entropy_tracking() {
        let mut state = RuntimeState::new(
            42,
            InitMode::Random,
            Preset::Organic,
            0,
            0,
            MouseInteractionMode::Disabled,
            3.0,
            IntensityMapping::linear(),
            &SimConfig::default(),
            PauseStyle::Vignette,
            false,
            false,
        );
        assert!(!state.track_entropy(5.0, 10.0, 5));
        assert!(state.track_entropy(15.0, 10.0, 1));
        state.reset_collapse_counter();
        assert_eq!(state.collapse_frame_counter, 0);
    }

    #[test]
    fn test_runtime_state_history() {
        let mut state = RuntimeState::new(
            42,
            InitMode::Random,
            Preset::Organic,
            0,
            0,
            MouseInteractionMode::Disabled,
            3.0,
            IntensityMapping::linear(),
            &SimConfig::default(),
            PauseStyle::Vignette,
            false,
            false,
        );
        state.update_history(60.0, 5.0, 0.5);
        assert_eq!(state.fps_history.len(), 1);
        for _ in 0..65 {
            state.update_history(60.0, 5.0, 0.5);
        }
        assert_eq!(state.fps_history.len(), 60);
    }

    #[test]
    fn test_runtime_state_actions() {
        let mut state = RuntimeState::new(
            42,
            InitMode::Random,
            Preset::Organic,
            0,
            0,
            MouseInteractionMode::Disabled,
            3.0,
            IntensityMapping::linear(),
            &SimConfig::default(),
            PauseStyle::Vignette,
            false,
            false,
        );

        state.toggle_pause();
        assert!(state.is_paused);
        state.toggle_pause();
        assert!(!state.is_paused);

        state.toggle_controls();
        assert!(state.overlay_state.is_open(OverlayType::Controls));
        state.toggle_controls();
        assert!(!state.overlay_state.is_open(OverlayType::Controls));

        state.toggle_keyboard_hints();
        assert!(state.overlay_state.is_open(OverlayType::KeyboardHints));
        state.toggle_keyboard_hints();
        assert!(!state.overlay_state.is_open(OverlayType::KeyboardHints));

        state.toggle_preset_comparison(Preset::Network);
        assert!(state.overlay_state.is_open(OverlayType::PresetComparison));
        state.toggle_preset_comparison(Preset::Network);
        assert!(!state.overlay_state.is_open(OverlayType::PresetComparison));

        assert!(!state.any_overlay_open());
        state.overlay_state.open(OverlayType::Dashboard);
        assert!(state.any_overlay_open());
        state.close_all_overlays();
        assert!(!state.any_overlay_open());

        state.cycle_controls_category(true);
        assert_eq!(state.controls_category_idx, 1);
        state.cycle_controls_category(false);
        assert_eq!(state.controls_category_idx, 0);

        state.set_preset(Preset::Fire);
        assert_eq!(state.current_preset, Preset::Fire);

        state.adjust_time_scale(0.5);
        assert_eq!(state.time_scale, 1.5);

        state.cycle_palette(10);
        assert_eq!(state.palette_index, 1);
        state.cycle_palette_reverse(10);
        assert_eq!(state.palette_index, 0);

        state.toggle_dither();
        assert!(matches!(state.dither_mode, DitherMode::Ordered { .. }));
        state.toggle_dither();
        assert_eq!(state.dither_mode, DitherMode::None);

        state.cycle_dither_mode();
        assert!(matches!(state.dither_mode, DitherMode::Ordered { .. }));

        state.adjust_sensor_angle(10.0);
        assert!(state.sensor_angle > 22.5);

        use crate::simulation::config::DiffusionKernel;
        state.cycle_diffusion_kernel();
        assert_eq!(state.diffusion_kernel, DiffusionKernel::Mean3x3);

        state.cycle_mouse_mode();
        assert_eq!(state.mouse_mode, MouseInteractionMode::Attract);

        state.cycle_wind_direction();
        assert_eq!(state.wind_direction, WindDirection::North);
    }

    #[test]
    fn test_runtime_state_undo_redo() {
        let mut state = RuntimeState::new(
            42,
            InitMode::Random,
            Preset::Organic,
            0,
            0,
            MouseInteractionMode::Disabled,
            3.0,
            IntensityMapping::linear(),
            &SimConfig::default(),
            PauseStyle::Vignette,
            false,
            false,
        );
        let orig_angle = state.sensor_angle;
        state.force_checkpoint();
        state.adjust_sensor_angle(10.0);
        assert_ne!(state.sensor_angle, orig_angle);

        state.undo();
        assert_eq!(state.sensor_angle, orig_angle);

        state.redo();
        assert_ne!(state.sensor_angle, orig_angle);
    }

    #[test]
    fn test_palette_shift_speed_degrees() {
        assert_eq!(PaletteShiftSpeed::Off.degrees_per_second(), 0.0);
        assert_eq!(PaletteShiftSpeed::Slow.degrees_per_second(), 5.0);
        assert_eq!(PaletteShiftSpeed::Medium.degrees_per_second(), 15.0);
        assert_eq!(PaletteShiftSpeed::Fast.degrees_per_second(), 45.0);
    }

    #[test]
    fn test_motion_blur_cycling() {
        let mut state = create_test_runtime_state();

        assert_eq!(state.motion_blur_frames, 0);

        state.cycle_motion_blur();
        assert_eq!(state.motion_blur_frames, 3);

        state.cycle_motion_blur();
        assert_eq!(state.motion_blur_frames, 5);

        state.cycle_motion_blur();
        assert_eq!(state.motion_blur_frames, 7);

        state.cycle_motion_blur();
        assert_eq!(state.motion_blur_frames, 0);
    }

    #[test]
    fn test_randomize_params_updates() {
        let mut state = create_test_runtime_state();

        state.wind_direction = WindDirection::North;
        use crate::simulation::config::TerrainType;
        state.terrain_type = TerrainType::None;
        state.palette_index = 0;

        state.randomize_params();

        assert_eq!(state.wind_direction, WindDirection::North);
    }

    #[test]
    fn test_wind_direction_values() {
        assert!(WindDirection::None.to_wind().is_none());
        assert!(WindDirection::North.to_wind().is_some());
        assert!(WindDirection::Northeast.to_wind().is_some());
        assert!(WindDirection::East.to_wind().is_some());
        assert!(WindDirection::Southeast.to_wind().is_some());
        assert!(WindDirection::South.to_wind().is_some());
        assert!(WindDirection::Southwest.to_wind().is_some());
        assert!(WindDirection::West.to_wind().is_some());
        assert!(WindDirection::Northwest.to_wind().is_some());

        let north = WindDirection::North.to_wind().unwrap();
        assert_eq!(north.dx, 0.0);
        assert_eq!(north.dy, -1.0);
    }
}
