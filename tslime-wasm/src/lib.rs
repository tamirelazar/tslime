use crate::renderer::WebGlRenderer;
use std::cell::RefCell;
use std::rc::Rc;
use tslime::simulation::{
    config::{InitMode, SimConfig},
    Simulation,
};
use wasm_bindgen::prelude::*;

mod renderer;

#[wasm_bindgen]
pub struct TslimeWasm {
    simulation: Simulation,
    renderer: Rc<RefCell<WebGlRenderer>>,
    trail_buffer: Vec<f32>,
    animation_id: Option<i32>,
    running: bool,
    width: u32,
    height: u32,
}

#[wasm_bindgen]
impl TslimeWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(width: u32, height: u32, canvas_id: &str, seed: u64) -> Result<TslimeWasm, JsValue> {
        console_error_panic_hook::set_once();

        let mut config = SimConfig::default();
        let agent_count = ((width * height) as f32 * 0.01) as usize;
        if let Some(ref mut species) = config.species_configs.first_mut() {
            species.count = agent_count;
        }

        let simulation = Simulation::new(
            width as usize,
            height as usize,
            config,
            seed,
            InitMode::Random,
            0,
        );

        let renderer = WebGlRenderer::new(canvas_id)?;

        Ok(TslimeWasm {
            simulation,
            renderer,
            trail_buffer: Vec::new(),
            animation_id: None,
            running: false,
            width,
            height,
        })
    }

    pub fn step(&mut self) {
        self.simulation.update(1.0);
    }

    pub fn render(&mut self) {
        self.simulation.trail_map_blended(&mut self.trail_buffer);

        let renderer = self.renderer.borrow();
        renderer.update_texture(&self.trail_buffer);
        renderer.render(2.0, (0.2, 0.8, 1.0));
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
            42,
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
            42,
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
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}
