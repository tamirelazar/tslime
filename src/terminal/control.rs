use crate::cli::Palette;
use crate::render::dither::{DitherMatrix, DitherMode};
use crate::simulation::config::{DiffusionKernel, InitMode, Preset, TerrainType, Wind};
use crossterm::event::KeyEvent;

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

const ALL_PALETTES: [Palette; 14] = [
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
    AdjustTimeScale(f32),
    CyclePalette,
    CyclePaletteReverse,
    ToggleHelp,
    ToggleControls,
    CloseOverlays,
    ToggleDither,
    CycleDitherMode,
    AdjustDitherIntensity(f32),
    Quit,
    AdjustSensorAngle(f32),
    AdjustTurnAngle(f32),
    AdjustStepSize(f32),
    AdjustDecay(f32),
    AdjustDeposit(f32),
    CycleDiffusionKernel,
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
    ToggleStats,
    ShowConfigBrowser,
    ShowConfigSaveDialog,
    ConfigBrowserUp,
    ConfigBrowserDown,
    ConfigBrowserSelect,
    ConfigBrowserDelete,
    ConfigSaveConfirm,
    ConfigCancel,
    None,
}

#[derive(Debug, Clone)]
pub struct RuntimeState {
    pub is_paused: bool,
    pub show_help: bool,
    pub show_controls: bool,
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
    pub mouse_timeout: f32,
    pub sensor_angle: f32,
    pub turn_angle: f32,
    pub step_size: f32,
    pub decay_factor: f32,
    pub deposit_amount: f32,
    pub diffusion_kernel: DiffusionKernel,
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
}

impl RuntimeState {
    pub fn new(
        seed: u64,
        init_mode: InitMode,
        initial_preset: Preset,
        initial_palette_index: usize,
        show_help: bool,
        mouse_mode: MouseInteractionMode,
        mouse_timeout: f32,
    ) -> Self {
        Self {
            is_paused: false,
            show_help,
            show_controls: false,
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
            sensor_angle: 22.5,
            turn_angle: 45.0,
            step_size: 1.0,
            decay_factor: 0.5,
            deposit_amount: 5.0,
            diffusion_kernel: DiffusionKernel::Mean3x3,
            wind_direction: WindDirection::None,
            terrain_type: TerrainType::None,
            terrain_strength: 1.0,
            auto_normalize: false,
            motion_blur_frames: 0,
            max_brightness: 20.0,
            fast_mode_enabled: false,
            palette_shift_speed: PaletteShiftSpeed::Off,
            invert_palette: false,
            reverse_palette: false,
            show_stats: false,
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
        }
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn toggle_controls(&mut self) {
        self.show_controls = !self.show_controls;
    }

    pub fn any_overlay_open(&self) -> bool {
        self.show_help || self.show_controls || self.show_stats
    }

    pub fn close_all_overlays(&mut self) {
        self.show_help = false;
        self.show_controls = false;
        self.show_stats = false;
    }

    pub fn cycle_controls_category(&mut self, forward: bool) {
        if forward {
            self.controls_category_idx = (self.controls_category_idx + 1) % 5;
        } else {
            self.controls_category_idx = if self.controls_category_idx == 0 {
                4
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
        self.current_preset = preset;
    }

    pub fn adjust_time_scale(&mut self, delta: f32) {
        let new_scale = (self.time_scale + delta).clamp(0.5, 4.0);
        self.time_scale = new_scale;
    }

    pub fn cycle_palette(&mut self, num_palettes: usize) {
        self.palette_index = (self.palette_index + 1) % num_palettes;
    }

    pub fn cycle_palette_reverse(&mut self, num_palettes: usize) {
        if self.palette_index == 0 {
            self.palette_index = num_palettes - 1;
        } else {
            self.palette_index -= 1;
        }
    }

    pub fn current_palette(&self, palettes: &[Palette; 14]) -> Palette {
        palettes[self.palette_index].clone()
    }

    pub fn toggle_dither(&mut self) {
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
        let new_value = (self.sensor_angle + delta).clamp(5.0, 90.0);
        let at_bound = (new_value - self.sensor_angle).abs() < 0.01;
        self.sensor_angle = new_value;
        at_bound
    }

    pub fn adjust_turn_angle(&mut self, delta: f32) -> bool {
        let new_value = (self.turn_angle + delta).clamp(5.0, 90.0);
        let at_bound = (new_value - self.turn_angle).abs() < 0.01;
        self.turn_angle = new_value;
        at_bound
    }

    pub fn adjust_step_size(&mut self, delta: f32) -> bool {
        let new_value = (self.step_size + delta).clamp(0.5, 5.0);
        let at_bound = (new_value - self.step_size).abs() < 0.01;
        self.step_size = new_value;
        at_bound
    }

    pub fn adjust_decay(&mut self, delta: f32) -> bool {
        let new_value = (self.decay_factor + delta).clamp(0.5, 0.99);
        let at_bound = (new_value - self.decay_factor).abs() < 0.001;
        self.decay_factor = new_value;
        at_bound
    }

    pub fn adjust_deposit(&mut self, delta: f32) -> bool {
        let new_value = (self.deposit_amount + delta).clamp(1.0, 20.0);
        let at_bound = (new_value - self.deposit_amount).abs() < 0.01;
        self.deposit_amount = new_value;
        at_bound
    }

    pub fn cycle_diffusion_kernel(&mut self) {
        self.diffusion_kernel = match self.diffusion_kernel {
            DiffusionKernel::Mean3x3 => DiffusionKernel::Gaussian,
            DiffusionKernel::Gaussian => DiffusionKernel::Mean3x3,
        };
    }

    pub fn cycle_wind_direction(&mut self) {
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
        let new_value = (self.terrain_strength + delta).clamp(0.1, 5.0);
        let at_bound = (new_value - self.terrain_strength).abs() < 0.01;
        self.terrain_strength = new_value;
        at_bound
    }

    pub fn cycle_terrain_type(&mut self) {
        self.terrain_type = match self.terrain_type {
            TerrainType::None => TerrainType::Smooth,
            TerrainType::Smooth => TerrainType::Turbulent,
            TerrainType::Turbulent => TerrainType::Mixed,
            TerrainType::Mixed => TerrainType::None,
        };
    }

    pub fn toggle_auto_normalize(&mut self) {
        self.auto_normalize = !self.auto_normalize;
    }

    pub fn cycle_motion_blur(&mut self) {
        self.motion_blur_frames = match self.motion_blur_frames {
            0 => 3,
            3 => 5,
            5 => 7,
            7 => 0,
            _ => 0,
        };
    }

    pub fn adjust_max_brightness(&mut self, delta: f32) -> bool {
        let new_value = (self.max_brightness + delta).clamp(1.0, 100.0);
        let at_bound = (new_value - self.max_brightness).abs() < 0.01;
        self.max_brightness = new_value;
        at_bound
    }

    pub fn toggle_fast_mode(&mut self) {
        self.fast_mode_enabled = !self.fast_mode_enabled;
    }

    pub fn cycle_palette_shift_speed(&mut self) {
        self.palette_shift_speed = match self.palette_shift_speed {
            PaletteShiftSpeed::Off => PaletteShiftSpeed::Slow,
            PaletteShiftSpeed::Slow => PaletteShiftSpeed::Medium,
            PaletteShiftSpeed::Medium => PaletteShiftSpeed::Fast,
            PaletteShiftSpeed::Fast => PaletteShiftSpeed::Off,
        };
    }

    pub fn toggle_invert_palette(&mut self) {
        self.invert_palette = !self.invert_palette;
    }

    pub fn toggle_reverse_palette(&mut self) {
        self.reverse_palette = !self.reverse_palette;
    }

    pub fn toggle_stats(&mut self) {
        self.show_stats = !self.show_stats;
    }

    pub fn reset_to_defaults(&mut self) {
        self.sensor_angle = 22.5;
        self.turn_angle = 45.0;
        self.step_size = 1.0;
        self.decay_factor = 0.5;
        self.deposit_amount = 5.0;
        self.diffusion_kernel = DiffusionKernel::Mean3x3;
        self.wind_direction = WindDirection::None;
        self.terrain_type = TerrainType::None;
        self.terrain_strength = 1.0;
        self.auto_normalize = false;
        self.motion_blur_frames = 0;
        self.max_brightness = 20.0;
        self.fast_mode_enabled = false;
        self.palette_shift_speed = PaletteShiftSpeed::Off;
        self.invert_palette = false;
        self.reverse_palette = false;
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
        KeyCode::Char('+') | KeyCode::Char('=') => ControlAction::AdjustTimeScale(0.5),
        KeyCode::Char('-') | KeyCode::Char('_') => ControlAction::AdjustTimeScale(-0.5),
        KeyCode::Char('C') if key_event.modifiers.contains(KeyModifiers::SHIFT) => {
            ControlAction::CyclePaletteReverse
        }
        KeyCode::Char('c') => ControlAction::CyclePalette,
        KeyCode::Char('?') => ControlAction::ToggleHelp,
        KeyCode::Char('h') | KeyCode::Char('H') => ControlAction::ToggleControls,
        KeyCode::Esc => ControlAction::CloseOverlays,
        KeyCode::Char('d') | KeyCode::Char('D') => ControlAction::ToggleDither,
        KeyCode::Char('m') | KeyCode::Char('M') => ControlAction::CycleDitherMode,
        KeyCode::Char('[') | KeyCode::Char('{') => ControlAction::AdjustDitherIntensity(-0.1),
        KeyCode::Char(']') | KeyCode::Char('}') => ControlAction::AdjustDitherIntensity(0.1),
        KeyCode::Char('q') | KeyCode::Char('Q') => ControlAction::Quit,
        KeyCode::Tab => ControlAction::CycleOptionsCategory,
        KeyCode::Char('A') | KeyCode::Char('a') => {
            if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                ControlAction::AdjustSensorAngle(-1.0)
            } else {
                ControlAction::AdjustSensorAngle(1.0)
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
            false,
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
            false,
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
            false,
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
            false,
            MouseInteractionMode::Disabled,
            0.0,
        );

        state.sensor_angle = 90.0;
        state.invert_palette = true;
        state.reverse_palette = true;
        state.palette_shift_speed = PaletteShiftSpeed::Fast;

        state.reset_to_defaults();

        assert_eq!(state.sensor_angle, 22.5);
        assert!(!state.invert_palette);
        assert!(!state.reverse_palette);
        assert_eq!(state.palette_shift_speed, PaletteShiftSpeed::Off);
    }

    #[test]
    fn test_help_toggle() {
        let mut state = RuntimeState::new(
            42,
            crate::simulation::config::InitMode::Random,
            crate::simulation::config::Preset::Network,
            0,
            false,
            MouseInteractionMode::Disabled,
            0.0,
        );

        assert!(!state.show_help);

        state.toggle_help();
        assert!(state.show_help);

        state.toggle_help();
        assert!(!state.show_help);
    }

    #[test]
    fn test_controls_toggle() {
        let mut state = RuntimeState::new(
            42,
            crate::simulation::config::InitMode::Random,
            crate::simulation::config::Preset::Network,
            0,
            false,
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
            false,
            MouseInteractionMode::Disabled,
            0.0,
        );

        assert!(!state.any_overlay_open());

        state.show_help = true;
        assert!(state.any_overlay_open());

        state.show_help = false;
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
            false,
            MouseInteractionMode::Disabled,
            0.0,
        );

        state.show_help = true;
        state.show_controls = true;
        state.show_stats = true;

        state.close_all_overlays();

        assert!(!state.show_help);
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
            false,
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
            false,
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
            false,
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
}
