use crate::renderer::WebGlRenderer;
use std::cell::RefCell;
use std::rc::Rc;
use tslime::render::adaptive_brightness::AdaptiveBrightness;
use tslime::render::ansi::render_ansi_cells;
use tslime::render::charset::Charset;
use tslime::render::downsample::{downsample, DownsampledFrame};
use tslime::render::palette::{Palette, ALL_PALETTES};
use tslime::simulation::{
    config::{InitMode, SimConfig, PRESETS},
    Simulation,
};
use wasm_bindgen::prelude::*;

mod renderer;

#[wasm_bindgen]
pub struct TslimeWasm {
    simulation: Simulation,
    // `None` in headless / ANSI mode (empty canvas id): the sim is rendered to
    // an escape-sequence string instead of a WebGL canvas.
    renderer: Option<Rc<RefCell<WebGlRenderer>>>,
    trail_buffer: Vec<f32>,
    animation_id: Option<i32>,
    running: bool,
    width: u32,
    height: u32,
    seed: u64,
    // ANSI-mode rendering scratch: a reusable downsample target plus the
    // adaptive white-point tracker (matches the TUI's auto-normalize).
    frame: Option<DownsampledFrame>,
    adaptive: AdaptiveBrightness,
    palette: Palette,
}

#[wasm_bindgen]
impl TslimeWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32, canvas_id: &str, seed: u32) -> Result<TslimeWasm, JsValue> {
        console_error_panic_hook::set_once();

        // Use the default population (50,000) — matching the TUI binary, whose
        // density gives the characteristic dense Physarum network. (The earlier
        // WebGL path scaled count to ~1% of the grid, which is far too sparse.)
        let config = SimConfig::default();

        let simulation = Simulation::new(
            width as usize,
            height as usize,
            config,
            seed as u64,
            InitMode::Random,
            0,
        );

        // Empty canvas id => headless ANSI mode (no WebGL context).
        let renderer = if canvas_id.is_empty() {
            None
        } else {
            Some(WebGlRenderer::new(canvas_id)?)
        };

        Ok(TslimeWasm {
            simulation,
            renderer,
            trail_buffer: Vec::new(),
            animation_id: None,
            running: false,
            width,
            height,
            seed: seed as u64,
            frame: None,
            // window=100, enabled=true — the TUI's default auto-normalize.
            adaptive: AdaptiveBrightness::new(100, true),
            palette: Palette::Warm,
        })
    }

    pub fn step(&mut self) {
        self.simulation.update(1.0);
    }

    pub fn render(&mut self) {
        let Some(renderer) = self.renderer.as_ref() else {
            return;
        };
        self.simulation.trail_map_blended(&mut self.trail_buffer);
        let renderer = renderer.borrow();
        renderer.update_texture(&self.trail_buffer);
        renderer.render(2.0, (0.2, 0.8, 1.0));
    }

    /// Render one ANSI frame for a `cols`×`rows` terminal. The white point is
    /// tracked adaptively across frames (matching the TUI's auto-normalize), so
    /// no manual gain is needed. Independent of WebGL — works in headless mode.
    pub fn render_ansi_frame(&mut self, cols: usize, rows: usize) -> String {
        self.simulation.trail_map_blended(&mut self.trail_buffer);

        // (Re)allocate the downsample target if the terminal grid changed.
        let need_new = match self.frame.as_ref() {
            Some(f) => f.width() != cols || f.height() != rows,
            None => true,
        };
        if need_new {
            self.frame = Some(DownsampledFrame::new(cols, rows));
        }
        let frame = self.frame.as_mut().unwrap();
        downsample(
            &self.trail_buffer,
            self.width as usize,
            self.height as usize,
            cols,
            rows,
            frame,
        );

        // Update the adaptive white point from this frame, then use it as the
        // brightness divisor — identical recipe to the TUI print path.
        self.adaptive.update(frame.cells());
        let gain = self.adaptive.get_max_brightness();

        // Bolt info-route launch look: warm palette, ASCII charset.
        render_ansi_cells(
            frame.cells(),
            cols,
            rows,
            self.palette,
            Charset::Ascii,
            gain,
        )
    }

    pub fn tick(&mut self) {
        self.step();
        self.render();
    }

    pub fn start(&mut self) {
        self.running = true;
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn set_agent_count(&mut self, count: usize) {
        let mut config = self.simulation.config().clone();
        if let Some(ref mut species) = config.species_configs.first_mut() {
            species.count = count;
        }
        self.simulation = Simulation::new(
            self.width as usize,
            self.height as usize,
            config,
            self.seed,
            InitMode::Random,
            0,
        );
    }

    pub fn set_config(
        &mut self,
        sensor_angle: f32,
        sensor_distance: f32,
        rotation_angle: f32,
        step_size: f32,
        decay: f32,
    ) {
        let mut config = self.simulation.config().clone();
        config.sensor_angle = sensor_angle;
        config.sensor_distance = sensor_distance;
        config.rotation_angle = rotation_angle;
        config.step_size = step_size;
        config.decay_factor = decay;

        self.simulation = Simulation::new(
            self.width as usize,
            self.height as usize,
            config,
            self.seed,
            InitMode::Random,
            0,
        );
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn get_trail_ptr(&self) -> *const f32 {
        self.trail_buffer.as_ptr()
    }

    pub fn get_trail_len(&self) -> usize {
        self.trail_buffer.len()
    }

    /// Set the active render palette by `ALL_PALETTES` index. Render-only —
    /// does NOT recreate the simulation. Out-of-range ids are ignored.
    pub fn set_palette(&mut self, id: u32) {
        if let Some(&p) = ALL_PALETTES.get(id as usize) {
            self.palette = p;
        }
    }

    pub fn palette_count(&self) -> u32 {
        ALL_PALETTES.len() as u32
    }

    pub fn palette_name(&self, id: u32) -> String {
        ALL_PALETTES
            .get(id as usize)
            .map(|p| p.name().to_string())
            .unwrap_or_default()
    }

    /// Apply a preset's *simulation* config by `PRESETS` index, reusing the
    /// original seed. NOTE: render look (palette/charset/effects) is NOT applied
    /// here — palette is the separate `set_palette` control. Out-of-range: no-op.
    pub fn set_preset(&mut self, id: u32) {
        let Some(spec) = PRESETS.get(id as usize) else {
            return;
        };
        let config: SimConfig = spec.preset.into();
        self.simulation = Simulation::new(
            self.width as usize,
            self.height as usize,
            config,
            self.seed,
            InitMode::Random,
            0,
        );
    }

    pub fn preset_count(&self) -> u32 {
        PRESETS.len() as u32
    }

    pub fn preset_name(&self, id: u32) -> String {
        PRESETS
            .get(id as usize)
            .map(|spec| spec.preset.name().to_string())
            .unwrap_or_default()
    }
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
