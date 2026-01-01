use crate::cli::Palette;
use crate::simulation::config::InitMode;
use crate::simulation::config::Preset;
use crossterm::event::KeyEvent;

const ALL_PALETTES: [Palette; 13] = [
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
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ControlAction {
    TogglePause,
    Restart,
    SetPreset(Preset),
    AdjustTimeScale(f32),
    CyclePalette,
    CyclePaletteReverse,
    ToggleHelp,
    Quit,
    None,
}

#[derive(Debug, Clone)]
pub struct RuntimeState {
    pub is_paused: bool,
    pub show_help: bool,
    pub time_scale: f32,
    pub current_preset: Preset,
    pub palette_index: usize,
    pub original_seed: u64,
    pub original_init_mode: InitMode,
}

impl RuntimeState {
    pub fn new(
        seed: u64,
        init_mode: InitMode,
        initial_preset: Preset,
        initial_palette_index: usize,
        show_help: bool,
    ) -> Self {
        Self {
            is_paused: false,
            show_help,
            time_scale: 1.0,
            current_preset: initial_preset,
            palette_index: initial_palette_index,
            original_seed: seed,
            original_init_mode: init_mode,
        }
    }

    pub fn toggle_pause(&mut self) {
        self.is_paused = !self.is_paused;
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
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

    pub fn current_palette(&self, palettes: &[Palette; 13]) -> Palette {
        palettes[self.palette_index].clone()
    }
}

pub fn num_palettes() -> usize {
    ALL_PALETTES.len()
}

pub fn handle_key_event(key_event: &KeyEvent) -> ControlAction {
    use crossterm::event::{KeyCode, KeyModifiers};

    match key_event.code {
        KeyCode::Char('p') | KeyCode::Char('P') | KeyCode::Char(' ') => ControlAction::TogglePause,
        KeyCode::Char('r') | KeyCode::Char('R') => ControlAction::Restart,
        KeyCode::Char('1') => ControlAction::SetPreset(Preset::Network),
        KeyCode::Char('2') => ControlAction::SetPreset(Preset::Exploratory),
        KeyCode::Char('3') => ControlAction::SetPreset(Preset::Tendrils),
        KeyCode::Char('4') => ControlAction::SetPreset(Preset::Organic),
        KeyCode::Char('5') => ControlAction::SetPreset(Preset::Minimal),
        KeyCode::Char('+') | KeyCode::Char('=') => ControlAction::AdjustTimeScale(0.5),
        KeyCode::Char('-') | KeyCode::Char('_') => ControlAction::AdjustTimeScale(-0.5),
        KeyCode::Char('C') if key_event.modifiers.contains(KeyModifiers::SHIFT) => {
            ControlAction::CyclePaletteReverse
        }
        KeyCode::Char('c') => ControlAction::CyclePalette,
        KeyCode::Char('h') | KeyCode::Char('H') => ControlAction::ToggleHelp,
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => ControlAction::Quit,
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
    }
}
