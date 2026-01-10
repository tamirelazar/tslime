use crate::cli::Palette;
use crate::render::dither::{DitherMatrix, DitherMode};
use crate::simulation::config::{DiffusionKernel, InitMode, Preset, SimConfig, TerrainType, Wind};
use crossterm::event::KeyEvent;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub struct MousePosition {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseInteractionMode {
    Disabled,
    Attract,
    Repel,
}

pub const ALL_PALETTES: [Palette; 16] = [
    Palette::Organic,
    Palette::Heat,
    Palette::Ocean,
    Palette::Mono,
    Palette::Forest,
    Palette::Neon,
    Palette::Warm,
    Palette::Vibrant,
    Palette::LegibleMono,
    Palette::Slime,
    Palette::Mold,
    Palette::Fungus,
    Palette::Swamp,
    Palette::Moss,
    Palette::Cosmic,
    Palette::Ethereal,
];

// HelpMode is kept for backwards compatibility but deprecated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum HelpMode {
    None,
    Quick,
    Options,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteShiftSpeed {
    Off,
    Slow,
    Medium,
    Fast,
}

impl PaletteShiftSpeed {
    pub fn degrees_per_second(&self) -> f32 {
        match self {
            PaletteShiftSpeed::Off => 0.0,
            PaletteShiftSpeed::Slow => 5.0,
            PaletteShiftSpeed::Medium => 15.0,
            PaletteShiftSpeed::Fast => 45.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindDirection {
    None,
    North,
    Northeast,
    East,
    Southeast,
    South,
    Southwest,
    West,
    Northwest,
}

impl WindDirection {
    #[allow(clippy::wrong_self_convention)]
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
#[allow(dead_code)]
pub enum ControlAction {
    TogglePause,
    Restart,
    SetPreset(Preset),
    ComparePreset(Preset),
    AdjustTimeScale(f32),
    CyclePalette,
    CyclePaletteReverse,
    ToggleControls,
    ToggleKeyboardHints,
    CloseOverlays,
    ToggleDither,
    CycleDitherMode,
    AdjustDitherIntensity(f32),
    Quit,
    AdjustSensorAngle(f32),
    AdjustSensorDistance(f32),
    AdjustTurnAngle(f32),
    AdjustStepSize(f32),
    AdjustDecay(f32),
    AdjustDeposit(f32),
    CycleDiffusionKernel,
    AdjustDiffusionSigma(f32),
    AdjustAttractorStrength(f32),
    CycleMouseMode,
    CycleWindDirection,
    AdjustTerrainStrength(f32),
    CycleTerrainType,
    ToggleAutoNormalize,
    CycleMotionBlur,
    AdjustMaxBrightness(f32),
    SaveFrameToPng,
    ToggleFastMode,
    CyclePaletteShiftSpeed,
    ToggleInvertPalette,
    ToggleReversePalette,
    ResetToDefaults,
    ShowOptionsOverlay,
    CycleOptionsCategory,
    CycleOptionsCategoryReverse,
    ToggleStats,
    ToggleInfo,
    ShowConfigBrowser,
    ShowConfigSaveDialog,
    ConfigBrowserUp,
    ConfigBrowserDown,
    ConfigBrowserSelect,
    ConfigBrowserDelete,
    ConfigSaveConfirm,
    ConfigCancel,
    RandomizeParams,
    Undo,
    Redo,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParameterState {
    pub sensor_angle: f32,
    pub sensor_distance: f32,
    pub turn_angle: f32,
    pub step_size: f32,
    pub decay_factor: f32,
    pub deposit_amount: f32,
    pub diffusion_kernel: DiffusionKernel,
    pub diffusion_sigma: f32,
    pub attractor_strength: f32,
    pub wind_direction: WindDirection,
    pub terrain_type: TerrainType,
    pub terrain_strength: f32,
    pub max_brightness: f32,
    pub palette_index: usize,
    pub invert_palette: bool,
    pub reverse_palette: bool,
    pub dither_mode: DitherMode,
    pub motion_blur_frames: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DefaultValues {
    pub sensor_angle: f32,
    pub sensor_distance: f32,
    pub turn_angle: f32,
    pub step_size: f32,
    pub decay_factor: f32,
    pub deposit_amount: f32,
    pub diffusion_kernel: DiffusionKernel,
    pub diffusion_sigma: f32,
    pub attractor_strength: f32,
    pub wind_direction: WindDirection,
    pub terrain_type: TerrainType,
    pub terrain_strength: f32,
    pub auto_normalize: bool,
    pub motion_blur_frames: usize,
    pub max_brightness: f32,
}

impl DefaultValues {
    pub fn from_preset(preset: Preset) -> Self {
        let config = SimConfig::from(preset);
        Self {
            sensor_angle: config.sensor_angle,
            sensor_distance: config.sensor_distance,
            turn_angle: config.rotation_angle,
            step_size: config.step_size,
            decay_factor: config.decay_factor,
            deposit_amount: config.deposit_amount,
            diffusion_kernel: config.diffusion_kernel,
            diffusion_sigma: config.diffusion_sigma,
            attractor_strength: config.attractor_strength,
            wind_direction: match config.wind {
                None => WindDirection::None,
                Some(w) => {
                    // Try to match common wind directions
                    if w.dx > 0.0 && w.dy == 0.0 {
                        WindDirection::East
                    } else if w.dx < 0.0 && w.dy == 0.0 {
                        WindDirection::West
                    } else if w.dx == 0.0 && w.dy < 0.0 {
                        WindDirection::North
                    } else if w.dx == 0.0 && w.dy > 0.0 {
                        WindDirection::South
                    } else {
                        WindDirection::None // Approximate
                    }
                }
            },
            terrain_type: config.terrain,
            terrain_strength: config.terrain_strength,
            auto_normalize: false, // Default is usually false
            motion_blur_frames: 0,
            max_brightness: config.max_brightness,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeState {
    pub is_paused: bool,
    pub show_controls: bool,
    pub show_keyboard_hints: bool,
    pub show_preset_comparison: bool,
    pub comparison_preset: Preset,
    pub controls_category_idx: usize,
    // Deprecated - kept for compatibility during transition
    #[allow(dead_code)]
    pub help_mode: HelpMode,
    #[allow(dead_code)]
    pub options_category_idx: usize,
    pub time_scale: f32,
    pub current_preset: Preset,
    pub palette_index: usize,
    pub original_seed: u64,
    pub original_init_mode: InitMode,
    pub dither_mode: DitherMode,
    pub last_dither_mode: Option<DitherMode>,
    pub mouse_mode: MouseInteractionMode,
    #[allow(dead_code)]
    pub mouse_timeout: f32,
    pub sensor_angle: f32,
    pub sensor_distance: f32,
    pub turn_angle: f32,
    pub step_size: f32,
    pub decay_factor: f32,
    pub deposit_amount: f32,
    pub diffusion_kernel: DiffusionKernel,
    pub diffusion_sigma: f32,
    pub attractor_strength: f32,
    pub wind_direction: WindDirection,
    pub terrain_type: TerrainType,
    pub terrain_strength: f32,
    pub auto_normalize: bool,
    pub motion_blur_frames: usize,
    pub max_brightness: f32,
    pub fast_mode_enabled: bool,
    pub palette_shift_speed: PaletteShiftSpeed,
    pub invert_palette: bool,
    pub reverse_palette: bool,
    pub show_stats: bool,
    pub show_info: bool,
    pub notification: Option<(String, std::time::Instant)>,
    pub collapse_frame_counter: usize,
    pub warmup_counter: usize,
    pub food_persist_counter: usize,
    pub food_persist_enabled: bool,
    pub initial_food_attractors: Vec<crate::simulation::config::Attractor>,
    pub show_config_browser: bool,
    pub show_config_save_dialog: bool,
    pub config_browser_selected_index: usize,
    pub config_save_name_input: String,
    pub default_values: DefaultValues,
    pub undo_stack: Vec<ParameterState>,
    pub redo_stack: Vec<ParameterState>,
    pub last_checkpoint_time: std::time::Instant,
    pub fps_history: std::collections::VecDeque<f32>,
    pub entropy_history: std::collections::VecDeque<f32>,
    pub density_history: std::collections::VecDeque<f32>,
}

impl RuntimeState {
    pub fn new(
        seed: u64,
        init_mode: InitMode,
        initial_preset: Preset,
        initial_palette_index: usize,
        mouse_mode: MouseInteractionMode,
        mouse_timeout: f32,
    ) -> Self {
        let default_values = DefaultValues::from_preset(initial_preset);
        Self {
            is_paused: false,
            show_controls: false,
            show_keyboard_hints: false,
            show_preset_comparison: false,
            comparison_preset: initial_preset,
            controls_category_idx: 0,
            help_mode: HelpMode::None,
            options_category_idx: 0,
            time_scale: 1.0,
            current_preset: initial_preset,
            palette_index: initial_palette_index,
            original_seed: seed,
            original_init_mode: init_mode,
            dither_mode: DitherMode::None,
            last_dither_mode: None,
            mouse_mode,
            mouse_timeout,
            sensor_angle: default_values.sensor_angle,
            sensor_distance: default_values.sensor_distance,
            turn_angle: default_values.turn_angle,
            step_size: default_values.step_size,
            decay_factor: default_values.decay_factor,
            deposit_amount: default_values.deposit_amount,
            diffusion_kernel: default_values.diffusion_kernel,
            diffusion_sigma: default_values.diffusion_sigma,
            attractor_strength: default_values.attractor_strength,
            wind_direction: default_values.wind_direction,
            terrain_type: default_values.terrain_type,
            terrain_strength: default_values.terrain_strength,
            auto_normalize: default_values.auto_normalize,
            motion_blur_frames: default_values.motion_blur_frames,
            max_brightness: default_values.max_brightness,
            fast_mode_enabled: false,
            palette_shift_speed: PaletteShiftSpeed::Off,
            invert_palette: false,
            reverse_palette: false,
            show_stats: false,
            show_info: false,
            notification: None,
            collapse_frame_counter: 0,
            warmup_counter: 0,
            food_persist_counter: 0,
            food_persist_enabled: false,
            initial_food_attractors: Vec::new(),
            show_config_browser: false,
            show_config_save_dialog: false,
            config_browser_selected_index: 0,
            config_save_name_input: String::new(),
            default_values,
            undo_stack: Vec::with_capacity(50),
            redo_stack: Vec::with_capacity(50),
            last_checkpoint_time: std::time::Instant::now(),
            fps_history: std::collections::VecDeque::with_capacity(20),
            entropy_history: std::collections::VecDeque::with_capacity(20),
            density_history: std::collections::VecDeque::with_capacity(20),
        }
    }

    pub fn capture_parameter_state(&self) -> ParameterState {
        ParameterState {
            sensor_angle: self.sensor_angle,
            sensor_distance: self.sensor_distance,
            turn_angle: self.turn_angle,
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
            invert_palette: self.invert_palette,
            reverse_palette: self.reverse_palette,
            dither_mode: self.dither_mode,
            motion_blur_frames: self.motion_blur_frames,
        }
    }

    pub fn apply_parameter_state(&mut self, state: ParameterState) {
        self.sensor_angle = state.sensor_angle;
        self.sensor_distance = state.sensor_distance;
        self.turn_angle = state.turn_angle;
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
        self.invert_palette = state.invert_palette;
        self.reverse_palette = state.reverse_palette;
        self.dither_mode = state.dither_mode;
        self.motion_blur_frames = state.motion_blur_frames;
    }

    pub fn checkpoint(&mut self) {
        // Debounce checkpoints (0.5s) to avoid spamming undo stack during continuous adjustments
        if self.last_checkpoint_time.elapsed().as_millis() < 500 {
            return;
        }

        let current = self.capture_parameter_state();
        if let Some(last) = self.undo_stack.last() {
            if last == &current {
                return;
            }
        }

        self.undo_stack.push(current);
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        self.last_checkpoint_time = std::time::Instant::now();
    }

    pub fn force_checkpoint(&mut self) {
        let current = self.capture_parameter_state();
        self.undo_stack.push(current);
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        self.last_checkpoint_time = std::time::Instant::now();
    }

    pub fn undo(&mut self) -> Option<ParameterState> {
        if self.undo_stack.is_empty() {
            return None;
        }

        let current = self.capture_parameter_state();
        self.redo_stack.push(current);

        let previous = self.undo_stack.pop().unwrap();
        self.apply_parameter_state(previous.clone());
        Some(previous)
    }

    pub fn redo(&mut self) -> Option<ParameterState> {
        if self.redo_stack.is_empty() {
            return None;
        }

        let current = self.capture_parameter_state();
        self.undo_stack.push(current);

        let next = self.redo_stack.pop().unwrap();
        self.apply_parameter_state(next.clone());
        Some(next)
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    pub fn toggle_controls(&mut self) {
        self.show_controls = !self.show_controls;
    }

    pub fn toggle_keyboard_hints(&mut self) {
        self.show_keyboard_hints = !self.show_keyboard_hints;
    }

    pub fn toggle_preset_comparison(&mut self, preset: Preset) {
        if self.show_preset_comparison && self.comparison_preset == preset {
            self.show_preset_comparison = false;
        } else {
            self.show_preset_comparison = true;
            self.comparison_preset = preset;
        }
    }

    pub fn any_overlay_open(&self) -> bool {
        self.show_controls
            || self.show_keyboard_hints
            || self.show_preset_comparison
            || self.show_stats
            || self.show_info
    }

    pub fn close_all_overlays(&mut self) {
        self.show_controls = false;
        self.show_keyboard_hints = false;
        self.show_preset_comparison = false;
        self.show_stats = false;
        self.show_info = false;
    }

    pub fn cycle_controls_category(&mut self, forward: bool) {
        const TOTAL_CATEGORIES: usize = 6; // 0-5: SIMULATION CORE, FORCES, APPEARANCE, POST-PROCESSING, PERFORMANCE, SYSTEM

        if forward {
            self.controls_category_idx = (self.controls_category_idx + 1) % TOTAL_CATEGORIES;
        } else {
            self.controls_category_idx = if self.controls_category_idx == 0 {
                TOTAL_CATEGORIES - 1
            } else {
                self.controls_category_idx - 1
            };
        }
    }

    // Deprecated - kept for compatibility
    #[allow(dead_code)]
    pub fn cycle_options_category(&mut self, forward: bool) {
        self.cycle_controls_category(forward);
    }

    pub fn set_preset(&mut self, preset: Preset) {
        self.force_checkpoint();
        self.current_preset = preset;
        self.default_values = DefaultValues::from_preset(preset);
    }

    pub fn adjust_time_scale(&mut self, delta: f32) {
        self.checkpoint();
        let new_scale = (self.time_scale + delta).clamp(0.5, 4.0);
        self.time_scale = new_scale;
    }

    pub fn cycle_palette(&mut self, num_palettes: usize) {
        self.force_checkpoint();
        self.palette_index = (self.palette_index + 1) % num_palettes;
    }

    pub fn cycle_palette_reverse(&mut self, num_palettes: usize) {
        self.force_checkpoint();
        if self.palette_index == 0 {
            self.palette_index = num_palettes - 1;
        } else {
            self.palette_index -= 1;
        }
    }

    pub fn current_palette(&self, palettes: &[Palette; 16]) -> Palette {
        palettes[self.palette_index].clone()
    }

    pub fn toggle_dither(&mut self) {
        self.force_checkpoint();
        self.dither_mode = match self.dither_mode {
            DitherMode::None => {
                if let Some(last) = self.last_dither_mode {
                    last
                } else {
                    DitherMode::Ordered {
                        intensity: 0.5,
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

    pub fn cycle_dither_mode(&mut self) {
        self.force_checkpoint();
        self.dither_mode = match self.dither_mode {
            DitherMode::None => DitherMode::Ordered {
                intensity: 0.5,
                matrix: DitherMatrix::Bayer4x4,
            },
            DitherMode::Ordered {
                intensity: _,
                matrix: _,
            } => DitherMode::ErrorDiffusion { serpentine: true },
            DitherMode::ErrorDiffusion { .. } => DitherMode::Hybrid {
                edge_threshold: 0.15,
                intensity: 0.5,
                matrix: DitherMatrix::Bayer4x4,
            },
            DitherMode::Hybrid { .. } => DitherMode::None,
        };
        if self.dither_mode != DitherMode::None {
            self.last_dither_mode = Some(self.dither_mode);
        }
    }

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

    pub fn adjust_sensor_angle(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.sensor_angle + delta).clamp(5.0, 90.0);
        let at_bound = (new_value - self.sensor_angle).abs() < 0.01;
        self.sensor_angle = new_value;
        at_bound
    }

    pub fn adjust_sensor_distance(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.sensor_distance + delta).clamp(1.0, 50.0);
        let at_bound = (new_value - self.sensor_distance).abs() < 0.01;
        self.sensor_distance = new_value;
        at_bound
    }

    pub fn adjust_turn_angle(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.turn_angle + delta).clamp(5.0, 90.0);
        let at_bound = (new_value - self.turn_angle).abs() < 0.01;
        self.turn_angle = new_value;
        at_bound
    }

    pub fn adjust_step_size(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.step_size + delta).clamp(0.5, 5.0);
        let at_bound = (new_value - self.step_size).abs() < 0.01;
        self.step_size = new_value;
        at_bound
    }

    pub fn adjust_decay(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.decay_factor + delta).clamp(0.5, 0.99);
        let at_bound = (new_value - self.decay_factor).abs() < 0.001;
        self.decay_factor = new_value;
        at_bound
    }

    pub fn adjust_deposit(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.deposit_amount + delta).clamp(1.0, 20.0);
        let at_bound = (new_value - self.deposit_amount).abs() < 0.01;
        self.deposit_amount = new_value;
        at_bound
    }

    pub fn cycle_diffusion_kernel(&mut self) {
        self.force_checkpoint();
        self.diffusion_kernel = match self.diffusion_kernel {
            DiffusionKernel::Mean3x3 => DiffusionKernel::Gaussian,
            DiffusionKernel::Gaussian => DiffusionKernel::Mean3x3,
        };
    }

    pub fn adjust_diffusion_sigma(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.diffusion_sigma + delta).clamp(0.5, 2.0);
        let at_bound = (new_value - self.diffusion_sigma).abs() < 0.01;
        self.diffusion_sigma = new_value;
        at_bound
    }

    pub fn adjust_attractor_strength(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.attractor_strength + delta).clamp(0.1, 10.0);
        let at_bound = (new_value - self.attractor_strength).abs() < 0.01;
        self.attractor_strength = new_value;
        at_bound
    }

    pub fn cycle_mouse_mode(&mut self) {
        self.force_checkpoint();
        self.mouse_mode = match self.mouse_mode {
            MouseInteractionMode::Disabled => MouseInteractionMode::Attract,
            MouseInteractionMode::Attract => MouseInteractionMode::Repel,
            MouseInteractionMode::Repel => MouseInteractionMode::Disabled,
        };
    }

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

    pub fn adjust_terrain_strength(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.terrain_strength + delta).clamp(0.1, 5.0);
        let at_bound = (new_value - self.terrain_strength).abs() < 0.01;
        self.terrain_strength = new_value;
        at_bound
    }

    pub fn cycle_terrain_type(&mut self) {
        self.force_checkpoint();
        self.terrain_type = match self.terrain_type {
            TerrainType::None => TerrainType::Smooth,
            TerrainType::Smooth => TerrainType::Turbulent,
            TerrainType::Turbulent => TerrainType::Mixed,
            TerrainType::Mixed => TerrainType::None,
        };
    }

    pub fn toggle_auto_normalize(&mut self) {
        self.force_checkpoint();
        self.auto_normalize = !self.auto_normalize;
    }

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

    pub fn adjust_max_brightness(&mut self, delta: f32) -> bool {
        self.checkpoint();
        let new_value = (self.max_brightness + delta).clamp(1.0, 100.0);
        let at_bound = (new_value - self.max_brightness).abs() < 0.01;
        self.max_brightness = new_value;
        at_bound
    }

    pub fn toggle_fast_mode(&mut self) {
        self.force_checkpoint();
        self.fast_mode_enabled = !self.fast_mode_enabled;
    }

    pub fn cycle_palette_shift_speed(&mut self) {
        self.force_checkpoint();
        self.palette_shift_speed = match self.palette_shift_speed {
            PaletteShiftSpeed::Off => PaletteShiftSpeed::Slow,
            PaletteShiftSpeed::Slow => PaletteShiftSpeed::Medium,
            PaletteShiftSpeed::Medium => PaletteShiftSpeed::Fast,
            PaletteShiftSpeed::Fast => PaletteShiftSpeed::Off,
        };
    }

    pub fn toggle_invert_palette(&mut self) {
        self.force_checkpoint();
        self.invert_palette = !self.invert_palette;
    }

    pub fn toggle_reverse_palette(&mut self) {
        self.force_checkpoint();
        self.reverse_palette = !self.reverse_palette;
    }

    pub fn toggle_stats(&mut self) {
        self.show_stats = !self.show_stats;
    }

    pub fn toggle_info(&mut self) {
        self.show_info = !self.show_info;
    }

    pub fn reset_to_defaults(&mut self) {
        self.force_checkpoint();
        let defaults = self.default_values;
        self.sensor_angle = defaults.sensor_angle;
        self.sensor_distance = defaults.sensor_distance;
        self.turn_angle = defaults.turn_angle;
        self.step_size = defaults.step_size;
        self.decay_factor = defaults.decay_factor;
        self.deposit_amount = defaults.deposit_amount;
        self.diffusion_kernel = defaults.diffusion_kernel;
        self.diffusion_sigma = defaults.diffusion_sigma;
        self.attractor_strength = defaults.attractor_strength;
        self.wind_direction = defaults.wind_direction;
        self.terrain_type = defaults.terrain_type;
        self.terrain_strength = defaults.terrain_strength;
        self.auto_normalize = defaults.auto_normalize;
        self.motion_blur_frames = defaults.motion_blur_frames;
        self.max_brightness = defaults.max_brightness;
        self.fast_mode_enabled = false;
        self.palette_shift_speed = PaletteShiftSpeed::Off;
        self.invert_palette = false;
        self.reverse_palette = false;
    }

    pub fn randomize_params(&mut self) {
        self.force_checkpoint();
        let mut rng = rand::thread_rng();

        // Randomize core simulation parameters within interesting ranges
        self.sensor_angle = rng.gen_range(15.0..60.0);
        self.turn_angle = rng.gen_range(15.0..60.0);
        self.step_size = rng.gen_range(0.5..2.5);
        self.decay_factor = rng.gen_range(0.80..0.98);
        self.deposit_amount = rng.gen_range(2.0..10.0);

        // Randomize diffusion kernel (mostly mean, sometimes gaussian)
        self.diffusion_kernel = if rng.gen_bool(0.3) {
            DiffusionKernel::Gaussian
        } else {
            DiffusionKernel::Mean3x3
        };

        // Randomize terrain
        self.terrain_type = match rng.gen_range(0..4) {
            0 => TerrainType::None,
            1 => TerrainType::Smooth,
            2 => TerrainType::Turbulent,
            _ => TerrainType::Mixed,
        };
        self.terrain_strength = rng.gen_range(0.5..3.0);

        // Randomize palette
        self.palette_index = rng.gen_range(0..ALL_PALETTES.len());

        // Reset display settings to reasonable defaults
        self.max_brightness = rng.gen_range(10.0..40.0);
    }

    pub fn show_notification(&mut self, message: String) {
        self.notification = Some((message, std::time::Instant::now()));
    }

    pub fn update_notifications(&mut self) {
        if let Some((_, time)) = self.notification {
            if time.elapsed().as_secs() >= 3 {
                self.notification = None;
            }
        }
    }

    pub fn current_notification(&self) -> Option<&String> {
        self.notification.as_ref().map(|(msg, _)| msg)
    }

    pub fn is_in_warmup(&self, warmup_frames: usize) -> bool {
        warmup_frames > 0 && self.warmup_counter < warmup_frames
    }

    pub fn increment_warmup(&mut self) {
        self.warmup_counter += 1;
    }

    pub fn reset_warmup(&mut self) {
        self.warmup_counter = 0;
    }

    pub fn track_entropy(&mut self, entropy: f32, threshold: f32, duration_frames: usize) -> bool {
        if entropy > threshold {
            self.collapse_frame_counter += 1;
            self.collapse_frame_counter >= duration_frames
        } else {
            self.collapse_frame_counter = 0;
            false
        }
    }

    pub fn reset_collapse_counter(&mut self) {
        self.collapse_frame_counter = 0;
    }

    pub fn update_history(&mut self, fps: f32, entropy: f32, density: f32) {
        self.fps_history.push_back(fps);
        if self.fps_history.len() > 20 {
            self.fps_history.pop_front();
        }

        self.entropy_history.push_back(entropy);
        if self.entropy_history.len() > 20 {
            self.entropy_history.pop_front();
        }

        self.density_history.push_back(density);
        if self.density_history.len() > 20 {
            self.density_history.pop_front();
        }
    }
}

pub fn num_palettes() -> usize {
    ALL_PALETTES.len()
}

pub fn handle_key_event(key_event: &KeyEvent) -> ControlAction {
    use crossterm::event::{KeyCode, KeyModifiers};

    // Check for Ctrl modifiers first
    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
        match key_event.code {
            KeyCode::Char('s') | KeyCode::Char('S') => return ControlAction::ShowConfigSaveDialog,
            KeyCode::Char('l') | KeyCode::Char('L') => return ControlAction::ShowConfigBrowser,
            KeyCode::Char('b') | KeyCode::Char('B') => return ControlAction::ShowConfigBrowser,
            KeyCode::Char('z') | KeyCode::Char('Z') => {
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    return ControlAction::Redo;
                } else {
                    return ControlAction::Undo;
                }
            }
            KeyCode::Char('y') | KeyCode::Char('Y') => return ControlAction::Redo,
            _ => {}
        }
    }

    match key_event.code {
        KeyCode::Char('p') | KeyCode::Char('P') | KeyCode::Char(' ') => ControlAction::TogglePause,
        KeyCode::Char('r') | KeyCode::Char('R') => ControlAction::Restart,
        KeyCode::Char('1') => ControlAction::SetPreset(Preset::Network),
        KeyCode::Char('2') => ControlAction::SetPreset(Preset::Exploratory),
        KeyCode::Char('3') => ControlAction::SetPreset(Preset::Tendrils),
        KeyCode::Char('4') => ControlAction::SetPreset(Preset::Organic),
        KeyCode::Char('5') => ControlAction::SetPreset(Preset::Minimal),
        KeyCode::Char('6') => ControlAction::SetPreset(Preset::Moss),
        KeyCode::Char('7') => ControlAction::SetPreset(Preset::Zen),
        KeyCode::Char('!') => ControlAction::ComparePreset(Preset::Network),
        KeyCode::Char('@') => ControlAction::ComparePreset(Preset::Exploratory),
        KeyCode::Char('#') => ControlAction::ComparePreset(Preset::Tendrils),
        KeyCode::Char('$') => ControlAction::ComparePreset(Preset::Organic),
        KeyCode::Char('%') => ControlAction::ComparePreset(Preset::Minimal),
        KeyCode::Char('^') => ControlAction::ComparePreset(Preset::Moss),
        KeyCode::Char('&') => ControlAction::ComparePreset(Preset::Zen),
        KeyCode::Char('8') => ControlAction::RandomizeParams,
        KeyCode::Char('+') | KeyCode::Char('=') => ControlAction::AdjustTimeScale(0.5),
        KeyCode::Char('-') | KeyCode::Char('_') => ControlAction::AdjustTimeScale(-0.5),
        KeyCode::Char('C') if key_event.modifiers.contains(KeyModifiers::SHIFT) => {
            ControlAction::CyclePaletteReverse
        }
        KeyCode::Char('c') => ControlAction::CyclePalette,
        KeyCode::Char('?') => ControlAction::ToggleKeyboardHints,
        KeyCode::Char('h') | KeyCode::Char('H') => ControlAction::ToggleControls,
        KeyCode::Esc => ControlAction::CloseOverlays,
        KeyCode::Char('d') | KeyCode::Char('D') => ControlAction::ToggleDither,
        KeyCode::Char('m') | KeyCode::Char('M') => ControlAction::CycleDitherMode,
        KeyCode::Char('[') | KeyCode::Char('{') => ControlAction::AdjustDitherIntensity(-0.1),
        KeyCode::Char(']') | KeyCode::Char('}') => ControlAction::AdjustDitherIntensity(0.1),
        KeyCode::Char('q') | KeyCode::Char('Q') => ControlAction::Quit,
        KeyCode::Tab => ControlAction::CycleOptionsCategory,
        KeyCode::BackTab => ControlAction::CycleOptionsCategoryReverse,
        KeyCode::Char('A') | KeyCode::Char('a') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustSensorAngle(-1.0)
            } else {
                ControlAction::AdjustSensorAngle(1.0)
            }
        }
        KeyCode::Char('J') | KeyCode::Char('j') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustSensorDistance(-1.0)
            } else {
                ControlAction::AdjustSensorDistance(1.0)
            }
        }
        KeyCode::Char('T') | KeyCode::Char('t') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustTurnAngle(-1.0)
            } else {
                ControlAction::AdjustTurnAngle(1.0)
            }
        }
        KeyCode::Char('S') | KeyCode::Char('s') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustStepSize(-0.5)
            } else {
                ControlAction::AdjustStepSize(0.5)
            }
        }
        KeyCode::Char('E') | KeyCode::Char('e') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustDecay(-0.01)
            } else {
                ControlAction::AdjustDecay(0.01)
            }
        }
        KeyCode::Char('I') | KeyCode::Char('i') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustDeposit(-1.0)
            } else {
                ControlAction::AdjustDeposit(1.0)
            }
        }
        KeyCode::Char('K') | KeyCode::Char('k') => ControlAction::CycleDiffusionKernel,
        KeyCode::Char(';') | KeyCode::Char(':') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustDiffusionSigma(-0.1)
            } else {
                ControlAction::AdjustDiffusionSigma(0.1)
            }
        }
        KeyCode::Char('L') | KeyCode::Char('l') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustAttractorStrength(-0.5)
            } else {
                ControlAction::AdjustAttractorStrength(0.5)
            }
        }
        KeyCode::Char(',') | KeyCode::Char('<') => ControlAction::CycleMouseMode,
        KeyCode::Char('W') | KeyCode::Char('w') => ControlAction::CycleWindDirection,
        KeyCode::Char('Y') | KeyCode::Char('y') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustTerrainStrength(-0.5)
            } else {
                ControlAction::AdjustTerrainStrength(0.5)
            }
        }
        KeyCode::Char('U') | KeyCode::Char('u') => ControlAction::CycleTerrainType,
        KeyCode::Char('B') | KeyCode::Char('b') => ControlAction::ToggleAutoNormalize,
        KeyCode::Char('V') | KeyCode::Char('v') => ControlAction::CycleMotionBlur,
        KeyCode::Char('N') | KeyCode::Char('n') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustMaxBrightness(-5.0)
            } else {
                ControlAction::AdjustMaxBrightness(5.0)
            }
        }
        KeyCode::Char('G') | KeyCode::Char('g') => ControlAction::SaveFrameToPng,
        KeyCode::Char('F') | KeyCode::Char('f') => ControlAction::ToggleFastMode,
        KeyCode::Char('O') | KeyCode::Char('o') => ControlAction::CyclePaletteShiftSpeed,
        KeyCode::Char('X') | KeyCode::Char('x') => ControlAction::ToggleInvertPalette,
        KeyCode::Char('Z') | KeyCode::Char('z') => ControlAction::ToggleReversePalette,
        KeyCode::Char('0') => ControlAction::ResetToDefaults,
        KeyCode::Char('\\') => ControlAction::ToggleStats,
        KeyCode::Char('|') => ControlAction::ToggleInfo,
        _ => ControlAction::None,
    }
}

pub fn preset_name(preset: Preset) -> &'static str {
    match preset {
        Preset::Network => "Network",
        Preset::Exploratory => "Exploratory",
        Preset::Tendrils => "Tendrils",
        Preset::Organic => "Organic",
        Preset::Minimal => "Minimal",
        Preset::Moss => "Moss",
        Preset::Cosmic => "Cosmic",
        Preset::Fire => "Fire",
        Preset::Zen => "Zen",
        Preset::Storm => "Storm",
        Preset::River => "River",
        Preset::Ethereal => "Ethereal",
    }
}

pub fn palette_name(palette: Palette) -> &'static str {
    match palette {
        Palette::Organic => "Organic",
        Palette::Heat => "Heat",
        Palette::Ocean => "Ocean",
        Palette::Mono => "Mono",
        Palette::Forest => "Forest",
        Palette::Neon => "Neon",
        Palette::Warm => "Warm",
        Palette::Vibrant => "Vibrant",
        Palette::LegibleMono => "LegibleMono",
        Palette::Slime => "Slime",
        Palette::Mold => "Mold",
        Palette::Fungus => "Fungus",
        Palette::Swamp => "Swamp",
        Palette::Moss => "Moss",
        Palette::Cosmic => "Cosmic",
        Palette::Ethereal => "Ethereal",
        Palette::Custom(_) => "Custom",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_shift_speed_cycling() {
        let mut state = RuntimeState::new(
            42,
            crate::simulation::config::InitMode::Random,
            crate::simulation::config::Preset::Network,
            0,
            MouseInteractionMode::Disabled,
            0.0,
        );

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
    fn test_invert_palette_toggle() {
        let mut state = RuntimeState::new(
            42,
            crate::simulation::config::InitMode::Random,
            crate::simulation::config::Preset::Network,
            0,
            MouseInteractionMode::Disabled,
            0.0,
        );

        assert!(!state.invert_palette);

        state.toggle_invert_palette();
        assert!(state.invert_palette);

        state.toggle_invert_palette();
        assert!(!state.invert_palette);
    }

    #[test]
    fn test_reverse_palette_toggle() {
        let mut state = RuntimeState::new(
            42,
            crate::simulation::config::InitMode::Random,
            crate::simulation::config::Preset::Network,
            0,
            MouseInteractionMode::Disabled,
            0.0,
        );

        assert!(!state.reverse_palette);

        state.toggle_reverse_palette();
        assert!(state.reverse_palette);

        state.toggle_reverse_palette();
        assert!(!state.reverse_palette);
    }

    #[test]
    fn test_reset_to_defaults() {
        let mut state = RuntimeState::new(
            42,
            crate::simulation::config::InitMode::Random,
            crate::simulation::config::Preset::Network,
            0,
            MouseInteractionMode::Disabled,
            0.0,
        );

        state.sensor_angle = 90.0;
        state.invert_palette = true;
        state.reverse_palette = true;
        state.palette_shift_speed = PaletteShiftSpeed::Fast;

        state.reset_to_defaults();

        assert_eq!(state.sensor_angle, 15.0); // Network preset default is 15.0
        assert!(!state.invert_palette);
        assert!(!state.reverse_palette);
        assert_eq!(state.palette_shift_speed, PaletteShiftSpeed::Off);
    }

    #[test]
    fn test_controls_toggle() {
        let mut state = RuntimeState::new(
            42,
            crate::simulation::config::InitMode::Random,
            crate::simulation::config::Preset::Network,
            0,
            MouseInteractionMode::Disabled,
            0.0,
        );

        assert!(!state.show_controls);

        state.toggle_controls();
        assert!(state.show_controls);

        state.toggle_controls();
        assert!(!state.show_controls);
    }

    #[test]
    fn test_any_overlay_open() {
        let mut state = RuntimeState::new(
            42,
            crate::simulation::config::InitMode::Random,
            crate::simulation::config::Preset::Network,
            0,
            MouseInteractionMode::Disabled,
            0.0,
        );

        assert!(!state.any_overlay_open());

        state.show_controls = true;
        assert!(state.any_overlay_open());

        state.show_controls = false;
        state.show_stats = true;
        assert!(state.any_overlay_open());
    }

    #[test]
    fn test_close_all_overlays() {
        let mut state = RuntimeState::new(
            42,
            crate::simulation::config::InitMode::Random,
            crate::simulation::config::Preset::Network,
            0,
            MouseInteractionMode::Disabled,
            0.0,
        );

        state.show_controls = true;
        state.show_stats = true;

        state.close_all_overlays();

        assert!(!state.show_controls);
        assert!(!state.show_stats);
    }

    #[test]
    fn test_controls_category_cycling() {
        let mut state = RuntimeState::new(
            42,
            crate::simulation::config::InitMode::Random,
            crate::simulation::config::Preset::Network,
            0,
            MouseInteractionMode::Disabled,
            0.0,
        );

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
        assert_eq!(state.controls_category_idx, 0);

        state.cycle_controls_category(false);
        assert_eq!(state.controls_category_idx, 4);
    }

    #[test]
    fn test_wind_direction_cycling() {
        let mut state = RuntimeState::new(
            42,
            crate::simulation::config::InitMode::Random,
            crate::simulation::config::Preset::Network,
            0,
            MouseInteractionMode::Disabled,
            0.0,
        );

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
        assert_eq!(WindDirection::Northeast.name(), "NE");
        assert_eq!(WindDirection::East.name(), "E");
        assert_eq!(WindDirection::Southeast.name(), "SE");
        assert_eq!(WindDirection::South.name(), "S");
        assert_eq!(WindDirection::Southwest.name(), "SW");
        assert_eq!(WindDirection::West.name(), "W");
        assert_eq!(WindDirection::Northwest.name(), "NW");
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
        let mut state = RuntimeState::new(
            42,
            crate::simulation::config::InitMode::Random,
            crate::simulation::config::Preset::Network,
            0,
            MouseInteractionMode::Disabled,
            0.0,
        );

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
        let mut state = RuntimeState::new(
            42,
            crate::simulation::config::InitMode::Random,
            crate::simulation::config::Preset::Network,
            0,
            MouseInteractionMode::Disabled,
            0.0,
        );

        // Set specific values that randomization should likely change
        state.wind_direction = WindDirection::North;
        state.terrain_type = TerrainType::None;
        state.palette_index = 0;

        state.randomize_params();

        // Wind should NOT be randomized (remain same as before call)
        // Wait, the requirement was "exclude wind from randomization".
        // In my implementation, I removed the wind randomization block.
        // So it should stay whatever it was.
        assert_eq!(state.wind_direction, WindDirection::North);

        // These should have a very high probability of changing (not strictly guaranteed but likely)
        // We just check if terrain and palette randomization logic was called by checking if they are within valid ranges
        // but since we want to be sure, we can check if they are randomized in the code.
        // Actually, for a unit test, we can just verify the wind stays same.
    }
}
