//! Interactive simulation runner and main event loop.
//!
//! This module contains the main simulation runner that handles the interactive
//! terminal-based simulation with real-time user input, rendering, and overlay management.

use std::io::{self, Write};

use crate::app::{apply_random_config, extract_species_rgb_colors, REFERENCE_TIME_STEP};
use crate::cli::{self, Args, ColorMode, Mode, Palette};
use crate::config_manager;
use crate::export::GifExporter;
use crate::export::WebmExporter;
use crate::food_image::FOOD_IMAGE_PNG;
use crate::palette_manager;
use crate::render::adaptive_brightness::AdaptiveBrightness;
use crate::render::charset::Charset;
use crate::render::dither::DitherMode;
use crate::render::downsample::downsample;
use crate::render::grid::{GridRenderer, GridStyle};
use crate::render::options_overlay::ControlsOverlay;
use crate::render::overlay::{
    build_notification_panel, ConfigBrowserOverlay, ConfigSaveOverlay, DashboardOverlay,
    KeyboardHintsOverlay, OverlayRenderer, PauseOverlay, PresetComparisonOverlay, RenderedOverlay,
};
use crate::render::palette::{hex_to_rgb, palette_accent_color, RgbColor};
use crate::render::palette_editor::{
    EditorComponent, EditorMode, PaletteEditorOverlay, PaletteEditorState,
};
use crate::simulation::config::TerrainType;
use crate::simulation::config::{Attractor, DiffusionKernel, InitMode, Preset, SimConfig};
use crate::simulation::food::load_logo_from_memory;
use crate::simulation::Simulation;
use crate::terminal::control::num_palettes;
use crate::terminal::control::{
    charset_name, handle_key_event, palette_name, preset_name, ControlAction, MouseInteractionMode,
    PaletteShiftSpeed, RuntimeState, ALL_CHARSETS, ALL_PALETTES,
};
use crate::terminal::detection::{log_capabilities, TerminalCapabilities};
use crate::terminal::frame_buffer::FrameBuffer;
use crate::terminal::input::{InputPoller, MouseEventType};
use crate::terminal::renderer::TerminalRenderer;
use crate::terminal::screen::TerminalScreen;
use crate::terminal::signal::is_shutdown_requested;
use crate::terminal::timing::FrameTimer;
use crossterm::event::Event;
use memory_stats::memory_stats;

/// Data structure holding all overlay states for rendering.
#[derive(Default)]
#[allow(dead_code)]
#[allow(clippy::type_complexity)]
struct OverlayData {
    pause_logo: Option<RenderedOverlay>,
    pause_logo_pos: (usize, usize),
    controls: Option<RenderedOverlay>,
    controls_pos: (usize, usize),
    status: Option<(String, usize, Vec<(usize, RgbColor)>)>,
    notification: Option<(RenderedOverlay, usize, usize)>,
    dashboard: Option<RenderedOverlay>,
    dashboard_pos: (usize, usize),
    config_browser: Option<RenderedOverlay>,
    config_browser_pos: (usize, usize),
    config_save: Option<RenderedOverlay>,
    config_save_pos: (usize, usize),
    keyboard_hints: Option<RenderedOverlay>,
    keyboard_hints_pos: (usize, usize),
    preset_comparison: Option<RenderedOverlay>,
    preset_comparison_pos: (usize, usize),
    palette_editor: Option<RenderedOverlay>,
    palette_editor_pos: (usize, usize),
}

/// Updates food persistence attractors with fade-out effect.
///
/// Gradually reduces the strength of food attractors over time using
/// quadratic easing for a smooth fade-out effect.
fn update_food_persistence(sim: &mut Simulation, runtime_state: &mut RuntimeState, args: &Args) {
    if !runtime_state.food_persist_enabled
        || runtime_state.is_paused
        || args.food_persist_duration == 0
    {
        return;
    }

    runtime_state.food_persist_counter += 1;

    if runtime_state.food_persist_counter <= args.food_persist_duration {
        // Calculate fade factor using quadratic easing
        let progress =
            runtime_state.food_persist_counter as f32 / args.food_persist_duration as f32;
        let fade_factor: f32 = (1.0 - progress).powi(2); // Quadratic fade-out

        // Update attractor strengths
        let mut new_config = sim.config().clone();
        new_config.attractors.clear();

        for attractor in &runtime_state.initial_food_attractors {
            new_config.attractors.push(Attractor::new(
                attractor.x,
                attractor.y,
                attractor.strength * fade_factor,
            ));
        }

        sim.update_config(new_config);
    } else if runtime_state.food_persist_counter == args.food_persist_duration + 1 {
        // Remove all food attractors when duration expires
        let mut new_config = sim.config().clone();
        new_config.attractors.clear();
        sim.update_config(new_config);
    }
}

/// Checks if simulation should auto-reset based on entropy collapse.
///
/// Monitors entropy levels and resets the simulation if it collapses
/// (entropy stays below threshold for specified duration).
fn check_auto_reset(
    sim: &mut Simulation,
    runtime_state: &mut RuntimeState,
    args: &Args,
    entropy: f32,
    init_mode: InitMode,
) {
    if !args.auto_reset || runtime_state.is_paused {
        return;
    }

    let should_reset = runtime_state.track_entropy(
        entropy,
        args.collapse_entropy_threshold,
        args.collapse_duration_frames,
    );

    if should_reset {
        // Generate new seed
        let new_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Reset simulation
        sim.reset(new_seed, init_mode);
        runtime_state.reset_collapse_counter();
        runtime_state.reset_warmup();
        runtime_state.food_persist_counter = 0;
        runtime_state.show_notification(format!(
            "Simulation collapsed - restarting with seed {}",
            new_seed
        ));
    }
}

/// Builds all overlays for the current frame.
///
/// TODO: Integrate this function into run_simulation to replace inline overlay building.
/// This is currently unused but kept for future refactoring.
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
fn build_overlays(
    runtime_state: &mut RuntimeState,
    term_width: usize,
    term_height: usize,
    sim: &Simulation,
    current_palette: &cli::Palette,
    ui_accent: RgbColor,
    timer: &FrameTimer,
    start_time: &std::time::Instant,
    init_mode: InitMode,
    color_mode: ColorMode,
    charset: &Charset,
    args: &Args,
    entropy: f32,
    _trail_density: f32,
    blended_trail: &[f32],
    seed: u64,
) -> OverlayData {
    let mut overlays = OverlayData::default();

    // Calculate trail statistics for dashboard
    let trail_sum: f32 = blended_trail.iter().sum();
    let trail_capacity = (sim.width() * sim.height()) as f32 * 10.0;

    // Build preset comparison overlay
    overlays.preset_comparison =
        if runtime_state.show_preset_comparison && !runtime_state.show_dashboard {
            Some(PresetComparisonOverlay::build_overlay(
                runtime_state,
                runtime_state.comparison_preset,
            ))
        } else {
            None
        };
    overlays.preset_comparison_pos = if overlays.preset_comparison.is_some() {
        PresetComparisonOverlay::calculate_position(term_width, term_height)
    } else {
        (0, 0)
    };

    // Build palette editor overlay
    overlays.palette_editor =
        if runtime_state.show_palette_editor && !runtime_state.show_dashboard {
            runtime_state.palette_editor_state.as_ref().map(|s| {
                PaletteEditorOverlay::build_overlay(s, &runtime_state.panel_style, ui_accent)
            })
        } else {
            None
        };
    overlays.palette_editor_pos = if overlays.palette_editor.is_some() {
        PaletteEditorOverlay::calculate_position(term_width, term_height)
    } else {
        (0, 0)
    };

    // Calculate controls position
    overlays.controls_pos = ControlsOverlay::calculate_position(term_width, term_height);

    // Build keyboard hints overlay
    overlays.keyboard_hints = if runtime_state.show_keyboard_hints && !runtime_state.show_dashboard
    {
        Some(KeyboardHintsOverlay::build_overlay(ui_accent))
    } else {
        None
    };
    overlays.keyboard_hints_pos = if overlays.keyboard_hints.is_some() {
        KeyboardHintsOverlay::calculate_position(term_width, term_height)
    } else {
        (0, 0)
    };

    // Build controls overlay
    overlays.controls = if runtime_state.show_controls && !runtime_state.show_dashboard {
        Some(ControlsOverlay::build_overlay(
            runtime_state.controls_category_idx,
            runtime_state.sensor_angle,
            runtime_state.sensor_distance,
            runtime_state.rotation_angle,
            runtime_state.step_size,
            runtime_state.decay_factor,
            runtime_state.deposit_amount,
            runtime_state.time_scale,
            runtime_state.diffusion_kernel,
            runtime_state.diffusion_sigma,
            runtime_state.attractor_strength,
            match runtime_state.mouse_mode {
                MouseInteractionMode::Disabled => "Disabled",
                MouseInteractionMode::Attract => "Attract",
                MouseInteractionMode::Repel => "Repel",
            },
            runtime_state.mouse_timeout,
            runtime_state.wind_direction,
            runtime_state.terrain_type,
            runtime_state.terrain_strength,
            runtime_state.auto_normalize,
            runtime_state.motion_blur_frames,
            runtime_state.max_brightness,
            runtime_state.fast_mode_enabled,
            runtime_state.current_palette(&ALL_PALETTES).name(),
            charset_name(&runtime_state.current_charset()),
            runtime_state.palette_shift_speed,
            runtime_state.invert_palette,
            runtime_state.reverse_palette,
            runtime_state.dither_mode.name(),
            term_width,
            runtime_state.default_values,
            sim.agent_count(),
            ui_accent,
            runtime_state.current_theme_name(),
            &runtime_state.panel_style,
            runtime_state.shift_held,
            runtime_state.trail_age_enabled,
            runtime_state.trail_age_mode,
            runtime_state.trail_age_reverse,
            runtime_state.trail_delta_enabled,
            runtime_state.gradient_magnitude_enabled,
        ))
    } else {
        None
    };

    // Build status line
    let diffusion_kernel_name = match runtime_state.diffusion_kernel {
        DiffusionKernel::Mean3x3 => "Mean3x3",
        DiffusionKernel::Gaussian => "Gaussian",
    };
    let (status_line, status_colors) = OverlayRenderer::build_status_line(
        runtime_state.is_paused,
        runtime_state.current_preset,
        runtime_state.time_scale,
        current_palette.clone(),
        runtime_state.dither_mode,
        term_width,
        Some(sim.agent_count()),
        Some(diffusion_kernel_name),
        !runtime_state.undo_stack.is_empty(),
        !runtime_state.redo_stack.is_empty(),
        Some(ui_accent),
    );
    let status_x = OverlayRenderer::status_line_x(&status_line, term_width);
    overlays.status = if runtime_state.any_overlay_open() || runtime_state.is_paused {
        Some((status_line, status_x, status_colors))
    } else {
        None
    };

    // Build notification overlay
    let notification_overlay: Option<RenderedOverlay> = runtime_state
        .current_notification_full()
        .map(|(msg, level)| build_notification_panel(msg, level, &runtime_state.panel_style));
    overlays.notification = notification_overlay.map(|overlay| {
        let outer_w = overlay.lines.first().map_or(0, |l| l.chars().count());
        let notif_x = if outer_w < term_width {
            (term_width - outer_w) / 2
        } else {
            0
        };
        let notif_y = term_height.saturating_sub(5);
        (overlay, notif_x, notif_y)
    });

    // Build dashboard overlay
    overlays.dashboard = if runtime_state.show_dashboard {
        let elapsed = start_time.elapsed().as_secs_f32();
        let trail_max = blended_trail.iter().fold(0.0f32, |m, &v| v.max(m));
        let memory_mb = memory_stats()
            .map(|m| m.physical_mem as f32 / 1024.0 / 1024.0)
            .unwrap_or(0.0);
        let frame_time_ms = timer.last_frame_ms();
        let cpu_percent = (frame_time_ms / 33.333) * 100.0;

        let init_mode_name = match init_mode {
            InitMode::Random => "Random",
            InitMode::CentralBurst => "Central",
            InitMode::Circle => "Circle",
            InitMode::Gradient => "Gradient",
            InitMode::WaveFront => "Wave",
            InitMode::Spiral => "Spiral",
            InitMode::RandomClusters => "Clusters",
            InitMode::Food => "Food",
            InitMode::Petri => "Petri",
        };

        let color_mode_name = match color_mode {
            ColorMode::TrueColor => "TrueColor",
            ColorMode::Bits8 => "8",
            ColorMode::Bits16 => "16",
            ColorMode::Bits256 => "256",
        };

        let charset_str = match *charset {
            Charset::HalfBlock => "HalfBlock",
            Charset::HalfBlockDual => "HalfBlockDual",
            Charset::Ascii => "ASCII",
            Charset::Braille => "Braille",
            Charset::Quadrant => "Quadrant",
            Charset::Shade => "Shade",
            Charset::Points => "Points",
            Charset::Sculpted => "Sculpted",
            Charset::CustomAscii(_) => "Custom",
        };

        let food_source = if init_mode == InitMode::Food {
            Some(args.food.clone())
        } else {
            None
        };

        let current_palette = ALL_PALETTES[runtime_state.palette_index].clone();
        let pname = palette_name(current_palette.clone());
        let prname = preset_name(runtime_state.current_preset);
        let palette_colors: Vec<RgbColor> = (0..78)
            .map(|i| {
                crate::render::palette::map_brightness_rgb(
                    i as f32 / 77.0,
                    current_palette.clone(),
                    runtime_state.reverse_palette,
                    runtime_state.invert_palette,
                    0.0,
                    None,
                )
            })
            .collect();

        let current_config = sim.config();

        Some(DashboardOverlay::build_overlay(
            sim.agent_count(),
            trail_sum,
            trail_capacity,
            trail_max,
            entropy,
            timer.current_fps() as f32,
            timer.average_fps() as f32,
            timer.frame_count(),
            elapsed,
            sim.width(),
            sim.height(),
            sim.attractor_count(),
            sim.obstacle_count(),
            sim.species_count(),
            memory_mb,
            cpu_percent,
            runtime_state.is_paused,
            prname,
            pname,
            &palette_colors,
            term_width,
            term_height,
            init_mode_name,
            color_mode_name,
            charset_str,
            !args.simd_off,
            current_config.decay_factor,
            current_config.sensor_angle,
            seed,
            &food_source,
            args.warmup_frames,
            args.auto_reset,
            ui_accent,
            &runtime_state.panel_style,
        ))
    } else {
        None
    };
    overlays.dashboard_pos = DashboardOverlay::calculate_position(term_width, term_height);

    // Build config browser overlay
    overlays.config_browser = if runtime_state.show_config_browser && !runtime_state.show_dashboard
    {
        match config_manager::list_configs() {
            Ok(configs) => {
                // Clamp selected index to valid range
                runtime_state.config_browser_selected_index = runtime_state
                    .config_browser_selected_index
                    .min(configs.len().saturating_sub(1));
                Some(ConfigBrowserOverlay::build_overlay(
                    &configs,
                    runtime_state.config_browser_selected_index,
                ))
            }
            Err(_) => {
                runtime_state.show_notification("Failed to load configurations".to_string());
                runtime_state.show_config_browser = false;
                None
            }
        }
    } else {
        None
    };
    overlays.config_browser_pos = if overlays.config_browser.is_some() {
        ConfigBrowserOverlay::calculate_position(term_width, term_height)
    } else {
        (0, 0)
    };

    // Build config save dialog overlay
    overlays.config_save = if runtime_state.show_config_save_dialog && !runtime_state.show_dashboard
    {
        Some(ConfigSaveOverlay::build_overlay(
            &runtime_state.config_save_name_input,
        ))
    } else {
        None
    };
    overlays.config_save_pos = if overlays.config_save.is_some() {
        ConfigSaveOverlay::calculate_position(term_width, term_height)
    } else {
        (0, 0)
    };

    overlays
}

/// Runs the interactive simulation loop (Live or Screensaver mode).
///
/// Handles terminal setup, input processing, simulation updates, and rendering
/// loop.
#[allow(unused_assignments)]
pub fn run_simulation(
    sim: &mut Simulation,
    args: &Args,
    _mode: Mode,
    palette: cli::Palette,
    charset: Charset,
) -> io::Result<()> {
    let mut screen = TerminalScreen::new();
    screen.setup()?;

    let capabilities = TerminalCapabilities::detect();
    log_capabilities(&capabilities, args.verbose);

    let mouse_mode = if args.mouse_attract {
        MouseInteractionMode::Attract
    } else if args.mouse_repel {
        MouseInteractionMode::Repel
    } else {
        MouseInteractionMode::Disabled
    };

    if mouse_mode != MouseInteractionMode::Disabled && capabilities.supports_mouse_tracking {
        if let Err(e) = crate::terminal::enable_mouse_tracking() {
            eprintln!(
                "Warning: Failed to enable mouse tracking: {}. Mouse interaction disabled.",
                e
            );
        }
    }

    let color_mode = capabilities.auto_select_color_mode(args.color_mode().ok());

    let config = args.to_sim_config();
    let background_color = config.background_color.as_ref().and_then(|c| hex_to_rgb(c));

    let init_mode = args
        .init
        .or(config.preferred_init_mode)
        .unwrap_or(InitMode::Food);

    let mut renderer = TerminalRenderer::new(
        0,
        0,
        palette,
        charset.clone(),
        args.reverse_palette,
        args.invert_palette,
        color_mode,
        background_color,
    );
    let dither_mode = args.dither_mode().unwrap_or(DitherMode::None);
    renderer.set_dither_mode(dither_mode);
    renderer.set_ascii_contrast(args.ascii_contrast);
    let mut timer = FrameTimer::with_time_scale(args.fps, args.frame_delay, args.time_scale);
    timer.set_adaptive_fps(args.auto_fps);
    let input_poller = InputPoller::new();

    let (mut term_width, mut term_height) = screen.get_size()?;
    renderer.set_dimensions(term_width as usize, term_height as usize);

    let seed = args.seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });

    let initial_preset = args.preset.unwrap_or(Preset::Organic);
    let initial_palette = args.palette().unwrap_or(cli::Palette::Moss);
    let initial_palette_index = if let cli::Palette::Custom(_) = initial_palette {
        4 // Default to Forest for custom palettes
    } else {
        ALL_PALETTES
            .iter()
            .position(|p| *p == initial_palette)
            .unwrap_or(4)
    };

    let initial_intensity_mapping = args
        .intensity_mapping()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    renderer.set_intensity_mapping(Some(initial_intensity_mapping.clone()));

    let initial_charset_index = ALL_CHARSETS.iter().position(|c| c == &charset).unwrap_or(0);

    let mut runtime_state = RuntimeState::new(
        seed,
        init_mode,
        initial_preset,
        initial_palette_index,
        initial_charset_index,
        mouse_mode,
        args.mouse_timeout,
        initial_intensity_mapping,
        &config,
    );
    runtime_state.preload_pause_logo(term_width as usize, term_height as usize);
    runtime_state.dither_mode = dither_mode;
    runtime_state.show_dashboard = args.stats;
    runtime_state.trail_age_enabled = args.trail_age;
    runtime_state.trail_delta_enabled = args.trail_delta;
    runtime_state.gradient_magnitude_enabled = args.gradient_magnitude;
    runtime_state.gradient_strength = args.gradient_strength;
    runtime_state.trail_age_hue_range = args.trail_age_hue_range;
    runtime_state.trail_age_blend = args.trail_age_blend;
    runtime_state.trail_age_mode = match args.trail_age_mode.as_str() {
        "alternating" => crate::config_defaults::TrailAgeMode::Alternating,
        _ => crate::config_defaults::TrailAgeMode::Bidirectional,
    };
    runtime_state.trail_age_reverse = args.trail_age_reverse;
    runtime_state.trail_delta_strength = args.trail_delta_strength;
    if args.trail_age {
        sim.set_compute_trail_age(true);
    }
    if args.trail_delta {
        sim.set_compute_trail_delta(true);
    }
    if args.gradient_magnitude {
        sim.set_compute_gradient_magnitude(true);
    }
    renderer.set_dither_mode(dither_mode);
    let mut palette_editor_state: Option<PaletteEditorState> = None;

    // Initialize food persistence
    if args.food_persist && init_mode == InitMode::Food {
        runtime_state.food_persist_enabled = true;
        runtime_state.initial_food_attractors = Simulation::create_food_attractors(
            args.resolution.width,
            args.resolution.height,
            &args.food,
            args.food_invert,
            args.food_scale,
            args.food_persist_strength,
            0.3,
        );

        // Apply initial food attractors to simulation
        let mut new_config = sim.config().clone();
        new_config
            .attractors
            .extend(runtime_state.initial_food_attractors.clone());
        sim.update_config(new_config);
    }

    // let config = args.to_sim_config(); // Already parsed above
    if args.species_colors {
        let species_rgb_colors = extract_species_rgb_colors(&config);
        renderer.set_species_colors(true, species_rgb_colors);
    }

    let mut adaptive_brightness =
        AdaptiveBrightness::new(args.normalize_window, args.auto_normalize);
    let mut hue_offset: f32 = 0.0;

    let mut current_auto_normalize = args.auto_normalize;
    let mut _current_max_brightness = args.max_brightness.unwrap_or(100.0);

    // Apply initial randomization if requested
    if args.random {
        runtime_state.randomize_params();
        apply_random_config(
            &runtime_state,
            sim,
            &mut renderer,
            &ALL_PALETTES,
            &mut _current_max_brightness,
        );
    }

    let start_time = std::time::Instant::now();

    // Initialize grid renderer if enabled
    let mut grid_renderer = if args.grid {
        let grid_style = args.grid_style.parse().unwrap_or(GridStyle::Cross);
        let grid_color = hex_to_rgb(&args.grid_color).unwrap_or(RgbColor {
            r: 255,
            g: 255,
            b: 255,
        });
        let mut renderer = GridRenderer::new(
            grid_style,
            args.grid_size,
            grid_color,
            args.grid_opacity,
            args.grid_adaptive,
        );
        renderer.initialize(term_width as usize, term_height as usize);
        Some(renderer)
    } else {
        None
    };

    loop {
        if is_shutdown_requested() {
            break;
        }

        if screen.check_resize() {
            let (new_width, new_height) = screen.get_size()?;
            if (new_width != term_width || new_height != term_height)
                && new_width > 0
                && new_height > 0
            {
                term_width = new_width;
                term_height = new_height;
                renderer.set_dimensions(term_width as usize, term_height as usize);
                // Reinitialize grid with new dimensions
                if let Some(grid) = &mut grid_renderer {
                    grid.initialize(term_width as usize, term_height as usize);
                }
            }
        }

        if term_width == 0 || term_height == 0 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            continue;
        }

        let dt = timer.delta_time();

        // Clamp dt to avoid simulation instability during lag spikes (max 0.1s / 10 FPS)
        let dt = dt.min(0.1);

        // Check if we're in warmup phase
        let in_warmup = !args.skip_warmup && runtime_state.is_in_warmup(args.warmup_frames);

        // Calculate transition fade factor for smooth warmup→normal transition
        const WARMUP_SPEED_MULTIPLIER: f32 = 0.3; // 30% speed during warmup
        const TRANSITION_DURATION_FRAMES: usize = 30; // 1 second at 30 FPS

        let frames_since_warmup = runtime_state
            .warmup_counter
            .saturating_sub(args.warmup_frames);
        let in_transition = frames_since_warmup < TRANSITION_DURATION_FRAMES;

        // Fade factor: 0.0 during warmup, interpolates 0.0→1.0 over 30 frames, then 1.0
        let fade_factor = if in_warmup {
            0.0
        } else if in_transition {
            frames_since_warmup as f32 / TRANSITION_DURATION_FRAMES as f32
        } else {
            1.0
        };

        if !runtime_state.is_paused {
            timer.start_sim();

            // Apply smooth speed transition from 0.3x to 1.0x
            let speed_multiplier =
                WARMUP_SPEED_MULTIPLIER + (1.0 - WARMUP_SPEED_MULTIPLIER) * fade_factor;

            // Sanity check speed multiplier to ensure it stays within valid bounds [0.3, 1.0]
            // This prevents any floating point drift that could cause simulation instability
            let speed_multiplier = speed_multiplier.clamp(WARMUP_SPEED_MULTIPLIER, 1.0);

            let adjusted_dt = dt * speed_multiplier;
            sim.update(adjusted_dt / REFERENCE_TIME_STEP);

            // Harden warmup logic: Explicitly cap the counter to avoid runaway increment
            // Previously relied on in_transition boolean which could be fragile
            if !args.skip_warmup
                && runtime_state.warmup_counter < args.warmup_frames + TRANSITION_DURATION_FRAMES
            {
                runtime_state.increment_warmup();
            }

            timer.end_sim_start_render();
        } else {
            timer.start_sim();
            timer.end_sim_start_render();
        }

        let blended_trail = sim.trail_map_blended();
        let mut downsampled = downsample(
            &blended_trail,
            sim.width(),
            sim.height(),
            term_width as usize,
            term_height as usize,
        );

        // Compute auxiliary frame for trail age / temporal delta / gradient
        let aux_frame = if runtime_state.trail_age_enabled
            || runtime_state.trail_delta_enabled
            || runtime_state.gradient_magnitude_enabled
        {
            Some(crate::render::downsample::downsample_aux(
                if runtime_state.trail_age_enabled {
                    sim.trail_age()
                } else {
                    None
                },
                if runtime_state.trail_delta_enabled {
                    sim.trail_delta()
                } else {
                    None
                },
                if runtime_state.gradient_magnitude_enabled {
                    sim.gradient_magnitude()
                } else {
                    None
                },
                sim.width(),
                sim.height(),
                term_width as usize,
                term_height as usize,
            ))
        } else {
            None
        };
        renderer.set_visual_fx(
            aux_frame,
            runtime_state.trail_age_enabled,
            runtime_state.trail_delta_enabled,
            runtime_state.trail_age_hue_range,
            runtime_state.trail_age_blend,
            runtime_state.trail_delta_strength,
            runtime_state.gradient_magnitude_enabled,
            runtime_state.gradient_strength,
            runtime_state.trail_age_mode,
            runtime_state.trail_age_reverse,
        );

        let current_config = args.to_sim_config();

        adaptive_brightness.update(downsampled.cells());
        let mut max_brightness = if args.auto_normalize {
            adaptive_brightness.get_max_brightness()
        } else {
            current_config.max_brightness
        };

        // Apply warmup brightness multiplier with smooth transition
        // Uses the same fade_factor as speed transition for consistency
        if in_warmup || in_transition {
            // Brightness fade_factor: 1.0 during warmup, then 1.0→0.0 over 30 frames
            // Inverted from speed fade_factor since we want brightness to decrease
            let brightness_fade = 1.0 - fade_factor;

            // Interpolate between normal and warmup brightness
            let multiplier = 1.0 + (args.warmup_brightness_multiplier - 1.0) * brightness_fade;
            max_brightness *= multiplier;
        }

        let current_palette = runtime_state.current_palette(&ALL_PALETTES);

        let shift_degrees = runtime_state.palette_shift_speed.degrees_per_second();
        let is_off = runtime_state.palette_shift_speed == PaletteShiftSpeed::Off;

        if is_off && hue_offset.abs() > 0.05 {
            let lerp_factor = 3.0 * dt;
            hue_offset *= 1.0 - lerp_factor;

            if hue_offset.abs() < 0.1 {
                hue_offset = 0.0;
            }
        } else if !is_off {
            hue_offset += shift_degrees * dt;
            hue_offset %= 360.0;
        }

        renderer.set_hue_shift(hue_offset);

        // Help lines are no longer used (deprecated), passing None to renderer
        // The renderer handles None gracefully by skipping the overlay

        // Build preset comparison overlay (Shift+1-7 keys)
        let preset_comparison_lines: Option<RenderedOverlay> =
            if runtime_state.show_preset_comparison && !runtime_state.show_dashboard {
                Some(PresetComparisonOverlay::build_overlay(
                    &runtime_state,
                    runtime_state.comparison_preset,
                ))
            } else {
                None
            };
        let (preset_comparison_x, preset_comparison_y) = if preset_comparison_lines.is_some() {
            PresetComparisonOverlay::calculate_position(term_width as usize, term_height as usize)
        } else {
            (0, 0)
        };

        let accent = palette_accent_color(
            &current_palette,
            runtime_state.reverse_palette,
            runtime_state.invert_palette,
            0.0,
            Some(&runtime_state.intensity_mapping),
        );
        let palette_editor_overlay: Option<RenderedOverlay> = (runtime_state.show_palette_editor
            && !runtime_state.show_dashboard)
            .then(|| {
                palette_editor_state.as_ref().map(|s| {
                    PaletteEditorOverlay::build_overlay(s, &runtime_state.panel_style, accent)
                })
            })
            .flatten();
        let (palette_editor_x, palette_editor_y) = if palette_editor_overlay.is_some() {
            PaletteEditorOverlay::calculate_position(term_width as usize, term_height as usize)
        } else {
            (0, 0)
        };

        // Calculate controls position (centered)
        let (controls_x, controls_y) =
            ControlsOverlay::calculate_position(term_width as usize, term_height as usize);

        // Palette accent colour used for key-binding highlights and title badges.
        let ui_accent = palette_accent_color(
            &current_palette,
            runtime_state.reverse_palette,
            runtime_state.invert_palette,
            hue_offset,
            Some(&runtime_state.intensity_mapping),
        );

        // Build keyboard hints overlay (? key)
        let keyboard_hints_lines: Option<RenderedOverlay> =
            if runtime_state.show_keyboard_hints && !runtime_state.show_dashboard {
                Some(KeyboardHintsOverlay::build_overlay(ui_accent))
            } else {
                None
            };
        let (keyboard_hints_x, keyboard_hints_y) = if keyboard_hints_lines.is_some() {
            KeyboardHintsOverlay::calculate_position(term_width as usize, term_height as usize)
        } else {
            (0, 0)
        };

        // Build controls overlay (h key)
        let controls_lines: Option<RenderedOverlay> =
            if runtime_state.show_controls && !runtime_state.show_dashboard {
                Some(ControlsOverlay::build_overlay(
                    runtime_state.controls_category_idx,
                    runtime_state.sensor_angle,
                    runtime_state.sensor_distance,
                    runtime_state.rotation_angle,
                    runtime_state.step_size,
                    runtime_state.decay_factor,
                    runtime_state.deposit_amount,
                    runtime_state.time_scale,
                    runtime_state.diffusion_kernel,
                    runtime_state.diffusion_sigma,
                    runtime_state.attractor_strength,
                    match runtime_state.mouse_mode {
                        MouseInteractionMode::Disabled => "Disabled",
                        MouseInteractionMode::Attract => "Attract",
                        MouseInteractionMode::Repel => "Repel",
                    },
                    runtime_state.mouse_timeout,
                    runtime_state.wind_direction,
                    runtime_state.terrain_type,
                    runtime_state.terrain_strength,
                    runtime_state.auto_normalize,
                    runtime_state.motion_blur_frames,
                    runtime_state.max_brightness,
                    runtime_state.fast_mode_enabled,
                    runtime_state.current_palette(&ALL_PALETTES).name(),
                    charset_name(&runtime_state.current_charset()),
                    runtime_state.palette_shift_speed,
                    runtime_state.invert_palette,
                    runtime_state.reverse_palette,
                    runtime_state.dither_mode.name(),
                    term_width as usize,
                    runtime_state.default_values,
                    sim.agent_count(),
                    ui_accent,
                    runtime_state.current_theme_name(),
                    &runtime_state.panel_style,
                    runtime_state.shift_held,
                    runtime_state.trail_age_enabled,
                    runtime_state.trail_age_mode,
                    runtime_state.trail_age_reverse,
                    runtime_state.trail_delta_enabled,
                    runtime_state.gradient_magnitude_enabled,
                ))
            } else {
                None
            };

        // Build status line (shown when any overlay visible or paused)
        let diffusion_kernel_name = match runtime_state.diffusion_kernel {
            DiffusionKernel::Mean3x3 => "Mean3x3",
            DiffusionKernel::Gaussian => "Gaussian",
        };
        let (status_line, status_colors) = OverlayRenderer::build_status_line(
            runtime_state.is_paused,
            runtime_state.current_preset,
            runtime_state.time_scale,
            current_palette.clone(),
            runtime_state.dither_mode,
            term_width as usize,
            Some(sim.agent_count()),
            Some(diffusion_kernel_name),
            !runtime_state.undo_stack.is_empty(),
            !runtime_state.redo_stack.is_empty(),
            Some(ui_accent),
        );
        let status_x = OverlayRenderer::status_line_x(&status_line, term_width as usize);
        let status_data = if runtime_state.any_overlay_open() || runtime_state.is_paused {
            Some((status_line, status_x, status_colors))
        } else {
            None
        };

        let notification_overlay: Option<RenderedOverlay> = runtime_state
            .current_notification_full()
            .map(|(msg, level)| build_notification_panel(msg, level, &runtime_state.panel_style));
        let notification_data = notification_overlay.as_ref().map(|overlay| {
            let outer_w = overlay.lines.first().map_or(0, |l| l.chars().count());
            let notif_x = if outer_w < term_width as usize {
                (term_width as usize - outer_w) / 2
            } else {
                0
            };
            let notif_y = (term_height as usize).saturating_sub(5);
            (overlay, notif_x, notif_y)
        });

        // Dashboard overlay (merged stats + info)
        let entropy = DashboardOverlay::calculate_entropy(&blended_trail, 100);
        let trail_sum: f32 = blended_trail.iter().sum();
        let trail_capacity = (sim.width() * sim.height()) as f32 * 10.0;
        let trail_density = if trail_capacity > 0.0 {
            (trail_sum / trail_capacity).min(1.0)
        } else {
            0.0
        };

        runtime_state.update_history(timer.current_fps() as f32, entropy, trail_density);

        let dashboard_overlay: Option<RenderedOverlay> = if runtime_state.show_dashboard {
            let elapsed = start_time.elapsed().as_secs_f32();
            let trail_max = blended_trail.iter().fold(0.0f32, |m, &v| v.max(m));
            let memory_mb = memory_stats()
                .map(|m| m.physical_mem as f32 / 1024.0 / 1024.0)
                .unwrap_or(0.0);
            let frame_time_ms = timer.last_frame_ms();
            let cpu_percent = (frame_time_ms / 33.333) * 100.0;

            let init_mode_name = match init_mode {
                InitMode::Random => "Random",
                InitMode::CentralBurst => "Central",
                InitMode::Circle => "Circle",
                InitMode::Gradient => "Gradient",
                InitMode::WaveFront => "Wave",
                InitMode::Spiral => "Spiral",
                InitMode::RandomClusters => "Clusters",
                InitMode::Food => "Food",
                InitMode::Petri => "Petri",
            };

            let color_mode_name = match color_mode {
                ColorMode::TrueColor => "TrueColor",
                ColorMode::Bits8 => "8",
                ColorMode::Bits16 => "16",
                ColorMode::Bits256 => "256",
            };

            let charset_str = match charset {
                Charset::HalfBlock => "HalfBlock",
                Charset::HalfBlockDual => "HalfBlockDual",
                Charset::Ascii => "ASCII",
                Charset::Braille => "Braille",
                Charset::Quadrant => "Quadrant",
                Charset::Shade => "Shade",
                Charset::Points => "Points",
                Charset::Sculpted => "Sculpted",
                Charset::CustomAscii(_) => "Custom",
            };

            let food_source = if init_mode == InitMode::Food {
                Some(args.food.clone())
            } else {
                None
            };

            let current_palette = ALL_PALETTES[runtime_state.palette_index].clone();
            let pname = palette_name(current_palette.clone());
            let prname = preset_name(runtime_state.current_preset);
            let palette_colors: Vec<RgbColor> = (0..78)
                .map(|i| {
                    crate::render::palette::map_brightness_rgb(
                        i as f32 / 77.0,
                        current_palette.clone(),
                        runtime_state.reverse_palette,
                        runtime_state.invert_palette,
                        0.0,
                        None,
                    )
                })
                .collect();

            let current_config = sim.config();

            Some(DashboardOverlay::build_overlay(
                sim.agent_count(),
                trail_sum,
                trail_capacity,
                trail_max,
                entropy,
                timer.current_fps() as f32,
                timer.average_fps() as f32,
                timer.frame_count(),
                elapsed,
                sim.width(),
                sim.height(),
                sim.attractor_count(),
                sim.obstacle_count(),
                sim.species_count(),
                memory_mb,
                cpu_percent,
                runtime_state.is_paused,
                prname,
                pname,
                &palette_colors,
                term_width as usize,
                term_height as usize,
                init_mode_name,
                color_mode_name,
                charset_str,
                !args.simd_off,
                current_config.decay_factor,
                current_config.sensor_angle,
                seed,
                &food_source,
                args.warmup_frames,
                args.auto_reset,
                ui_accent,
                &runtime_state.panel_style,
            ))
        } else {
            None
        };

        let (dashboard_x, dashboard_y) =
            DashboardOverlay::calculate_position(term_width as usize, term_height as usize);

        // Config browser overlay
        let config_browser_overlay: Option<RenderedOverlay> = if runtime_state.show_config_browser
            && !runtime_state.show_dashboard
        {
            match config_manager::list_configs() {
                Ok(configs) => {
                    // Clamp selected index to valid range
                    runtime_state.config_browser_selected_index = runtime_state
                        .config_browser_selected_index
                        .min(configs.len().saturating_sub(1));
                    Some(ConfigBrowserOverlay::build_overlay(
                        &configs,
                        runtime_state.config_browser_selected_index,
                    ))
                }
                Err(_) => {
                    runtime_state.show_notification("Failed to load configurations".to_string());
                    runtime_state.show_config_browser = false;
                    None
                }
            }
        } else {
            None
        };
        let (config_browser_x, config_browser_y) = if config_browser_overlay.is_some() {
            ConfigBrowserOverlay::calculate_position(term_width as usize, term_height as usize)
        } else {
            (0, 0)
        };

        // Config save dialog overlay
        let config_save_overlay: Option<RenderedOverlay> =
            if runtime_state.show_config_save_dialog && !runtime_state.show_dashboard {
                Some(ConfigSaveOverlay::build_overlay(
                    &runtime_state.config_save_name_input,
                ))
            } else {
                None
            };
        let (config_save_x, config_save_y) = if config_save_overlay.is_some() {
            ConfigSaveOverlay::calculate_position(term_width as usize, term_height as usize)
        } else {
            (0, 0)
        };

        // Food persistence fade-out
        update_food_persistence(sim, &mut runtime_state, args);

        // Update focused overlay state before rendering
        runtime_state.update_focused_overlay();

        // Entropy-based auto-reset
        check_auto_reset(sim, &mut runtime_state, args, entropy, init_mode);

        // VCR pause overlays: dim logo + blinking badge
        let (pause_logo_overlay, pause_logo_x, pause_logo_y) = if runtime_state.is_paused {
            // Scale logo to terminal: small terminals use more %, large ones less
            let pct = if term_width < 80 {
                0.90
            } else if term_width < 120 {
                0.75
            } else {
                0.60
            };
            let logo_w = ((term_width as f32 * pct) as usize).clamp(30, 180);
            // Image is 1365×1024 (~1.33:1). Quadrant cells are 2×2 sub-pixels.
            // Terminal cell aspect ≈ 1:2 → logo_h = logo_w / (image_aspect * 2) = logo_w / 2.67
            let logo_h = ((logo_w as f32 / 2.67) as usize).max(6);
            // Quadrant: 2 pixels wide × 2 pixels tall per terminal cell
            let pixel_w = logo_w * 2;
            let pixel_h = logo_h * 2;

            // Decode or reuse cached brightness map
            let brightness_map = if runtime_state
                .pause_logo_cache
                .as_ref()
                .is_some_and(|(cw, _, _)| *cw == logo_w)
            {
                runtime_state.pause_logo_cache.as_ref().unwrap().2.clone()
            } else {
                let map = load_logo_from_memory(FOOD_IMAGE_PNG, pixel_w, pixel_h, true)
                    .unwrap_or_else(|_| vec![0.0; pixel_w * pixel_h]);
                runtime_state.pause_logo_cache = Some((logo_w, pixel_h, map.clone()));
                map
            };

            let current_palette = ALL_PALETTES[runtime_state.palette_index].clone();
            let logo_mapping = args
                .logo_mapping()
                .ok()
                .flatten()
                .unwrap_or_else(|| runtime_state.intensity_mapping.clone());
            let logo = PauseOverlay::build_logo(
                &brightness_map,
                logo_w,
                logo_h,
                current_palette,
                runtime_state.reverse_palette,
                runtime_state.invert_palette,
                0.0,
                Some(&logo_mapping),
            );

            runtime_state.pause_frame_counter += 1;

            let actual_logo_h = logo.lines.len();
            let lx = (term_width as usize).saturating_sub(logo_w) / 2;
            // Center vertically in the drawable area (exclude status bar row)
            let drawable_h = (term_height as usize).saturating_sub(1);
            let ly = drawable_h.saturating_sub(actual_logo_h) / 2;

            (Some(logo), lx, ly)
        } else {
            runtime_state.pause_frame_counter = 0;
            (None, 0, 0)
        };

        if args.species_colors && sim.config().separate_species_trails {
            let species_trail_maps = sim.trail_maps_for_species_colors();
            let species_rgb_colors = extract_species_rgb_colors(&current_config);
            let combined: Vec<_> = species_trail_maps
                .iter()
                .zip(species_rgb_colors.iter())
                .map(|(tm, color)| (*tm, *color))
                .collect();
            renderer.render_multi_species_with_overlay(
                &combined,
                sim.width(),
                sim.height(),
                max_brightness.max(1.0),
                if runtime_state.is_paused {
                    Some(runtime_state.pause_frame_counter)
                } else {
                    None
                },
                pause_logo_overlay
                    .as_ref()
                    .map(|v| (v, pause_logo_x, pause_logo_y)),
                None, // no badge — status bar shows PAUSED
                controls_lines.as_ref().map(|v| (v, controls_x, controls_y)),
                status_data,
                notification_data,
                dashboard_overlay
                    .as_ref()
                    .map(|v| (v, dashboard_x, dashboard_y)),
                grid_renderer.as_ref(),
                config_browser_overlay
                    .as_ref()
                    .map(|v| (v, config_browser_x, config_browser_y)),
                config_save_overlay
                    .as_ref()
                    .map(|v| (v, config_save_x, config_save_y)),
                keyboard_hints_lines
                    .as_ref()
                    .map(|v| (v, keyboard_hints_x, keyboard_hints_y)),
                preset_comparison_lines
                    .as_ref()
                    .map(|v| (v, preset_comparison_x, preset_comparison_y)),
                palette_editor_overlay
                    .as_ref()
                    .map(|v| (v, palette_editor_x, palette_editor_y)),
                Some(&runtime_state.panel_style),
                runtime_state.focused_overlay,
            )?;
        } else {
            renderer.render_with_overlay(
                downsampled.cells(),
                max_brightness.max(1.0),
                if runtime_state.is_paused {
                    Some(runtime_state.pause_frame_counter)
                } else {
                    None
                },
                pause_logo_overlay
                    .as_ref()
                    .map(|v| (v, pause_logo_x, pause_logo_y)),
                None, // no badge — status bar shows PAUSED
                controls_lines.as_ref().map(|v| (v, controls_x, controls_y)),
                status_data,
                notification_data,
                dashboard_overlay
                    .as_ref()
                    .map(|v| (v, dashboard_x, dashboard_y)),
                grid_renderer.as_ref(),
                config_browser_overlay
                    .as_ref()
                    .map(|v| (v, config_browser_x, config_browser_y)),
                config_save_overlay
                    .as_ref()
                    .map(|v| (v, config_save_x, config_save_y)),
                keyboard_hints_lines
                    .as_ref()
                    .map(|v| (v, keyboard_hints_x, keyboard_hints_y)),
                preset_comparison_lines
                    .as_ref()
                    .map(|v| (v, preset_comparison_x, preset_comparison_y)),
                palette_editor_overlay
                    .as_ref()
                    .map(|v| (v, palette_editor_x, palette_editor_y)),
                Some(&runtime_state.panel_style),
                runtime_state.focused_overlay,
            )?;
        }

        timer.end_render();

        let mut should_exit = false;
        let events = input_poller.drain_all_events()?;
        for event in events {
            match event {
                Event::Key(key_event) => {
                    // Track shift key state - only update on key press to avoid resetting on release
                    use crossterm::event::KeyEventKind;
                    if key_event.kind == KeyEventKind::Press {
                        use crossterm::event::KeyModifiers;
                        runtime_state.shift_held =
                            key_event.modifiers.contains(KeyModifiers::SHIFT);
                    }

                    // GLOBAL EXIT HANDLING -- always allow 'q' to quit regardless of overlay
                    if InputPoller::is_exit_key(&key_event) {
                        should_exit = true;
                        break;
                    }
                    let action = handle_key_event(&key_event);
                    if let ControlAction::Quit = action {
                        should_exit = true;
                        break;
                    }

                    // Skip warmup on any key press
                    if in_warmup {
                        runtime_state.warmup_counter = args.warmup_frames; // Skip to end
                    }

                    // Close keyboard hints on any key press
                    if runtime_state.show_keyboard_hints {
                        runtime_state.show_keyboard_hints = false;
                        continue;
                    }

                    // Handle config save dialog input
                    if runtime_state.show_config_save_dialog {
                        use crossterm::event::{KeyCode, KeyModifiers};
                        match key_event.code {
                            KeyCode::Char(c)
                                if !key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                            {
                                if runtime_state.config_save_name_input.len() < 26 {
                                    runtime_state.config_save_name_input.push(c);
                                }
                                continue;
                            }
                            KeyCode::Backspace => {
                                runtime_state.config_save_name_input.pop();
                                continue;
                            }
                            KeyCode::Enter => {
                                if !runtime_state.config_save_name_input.is_empty() {
                                    // Save config
                                    let saved_config = config_manager::SavedConfig::from_runtime(
                                        runtime_state.config_save_name_input.clone(),
                                        sim.config(),
                                        runtime_state.current_palette(&ALL_PALETTES),
                                        charset.clone(),
                                        args.reverse_palette,
                                        args.invert_palette,
                                        args.warmup_frames,
                                        args.food_persist,
                                        args.auto_reset,
                                        args.grid,
                                        if args.grid {
                                            Some(args.grid_style.clone())
                                        } else {
                                            None
                                        },
                                        init_mode,
                                        if init_mode == InitMode::Food {
                                            Some(args.food.clone())
                                        } else {
                                            None
                                        },
                                        Some(&runtime_state.intensity_mapping),
                                    );

                                    match config_manager::save_config(saved_config) {
                                        Ok(_) => {
                                            runtime_state.show_notification(format!(
                                                "Config '{}' saved successfully",
                                                runtime_state.config_save_name_input
                                            ));
                                        }
                                        Err(e) => {
                                            runtime_state.show_notification(format!(
                                                "Failed to save config: {}",
                                                e
                                            ));
                                        }
                                    }
                                }
                                runtime_state.show_config_save_dialog = false;
                                continue;
                            }
                            KeyCode::Esc => {
                                runtime_state.show_config_save_dialog = false;
                                continue;
                            }
                            _ => continue,
                        }
                    }

                    // Handle preset comparison input
                    if runtime_state.show_preset_comparison {
                        use crossterm::event::KeyCode;
                        match key_event.code {
                            KeyCode::Enter => {
                                // Apply the comparison preset
                                let preset = runtime_state.comparison_preset;
                                runtime_state.set_preset(preset);
                                let mut new_config = SimConfig::from(preset);
                                // Maintain existing environment/setup
                                new_config.attractors = sim.config().attractors.clone();
                                new_config.attractor_strength = sim.config().attractor_strength;
                                new_config.food_image_path = sim.config().food_image_path.clone();
                                new_config.food_image_invert = sim.config().food_image_invert;
                                new_config.obstacles = sim.config().obstacles.clone();
                                new_config.obstacle_masks = sim.config().obstacle_masks.clone();
                                sim.update_config(new_config);

                                runtime_state.show_notification(format!(
                                    "Applied preset: {}",
                                    crate::terminal::control::preset_name(preset)
                                ));
                                runtime_state.show_preset_comparison = false;
                                continue;
                            }
                            KeyCode::Esc => {
                                runtime_state.show_preset_comparison = false;
                                continue;
                            }
                            _ => {} // Allow other keys (like Shift+1-7 to switch preset being compared)
                        }
                    }

                    // Handle config browser input
                    if runtime_state.show_config_browser {
                        use crossterm::event::KeyCode;
                        match key_event.code {
                            KeyCode::Up => {
                                if runtime_state.config_browser_selected_index > 0 {
                                    runtime_state.config_browser_selected_index -= 1;
                                }
                                continue;
                            }
                            KeyCode::Down => {
                                // Will increment if there are configs available
                                runtime_state.config_browser_selected_index += 1;
                                continue;
                            }
                            KeyCode::Enter => {
                                // Load selected config
                                if let Ok(configs) = config_manager::list_configs() {
                                    if let Some(config) =
                                        configs.get(runtime_state.config_browser_selected_index)
                                    {
                                        match config.apply_to_runtime_state(&mut runtime_state) {
                                            Ok(_) => {
                                                // Update renderer with new visual parameters
                                                let new_palette =
                                                    runtime_state.current_palette(&ALL_PALETTES);
                                                renderer.set_palette(new_palette);
                                                renderer.set_invert_palette(
                                                    runtime_state.invert_palette,
                                                );
                                                renderer.set_reverse_palette(
                                                    runtime_state.reverse_palette,
                                                );

                                                runtime_state.show_notification(format!(
                                                    "Config '{}' loaded successfully",
                                                    config.name
                                                ));
                                            }
                                            Err(e) => {
                                                runtime_state.show_notification(format!(
                                                    "Failed to load '{}': {}",
                                                    config.name, e
                                                ));
                                            }
                                        }
                                    }
                                }
                                runtime_state.show_config_browser = false;
                                continue;
                            }
                            KeyCode::Delete => {
                                // Delete selected config
                                if let Ok(configs) = config_manager::list_configs() {
                                    if let Some(config) =
                                        configs.get(runtime_state.config_browser_selected_index)
                                    {
                                        let name = config.name.clone();
                                        match config_manager::delete_config(&name) {
                                            Ok(_) => {
                                                runtime_state.show_notification(format!(
                                                    "Deleted config '{}'",
                                                    name
                                                ));
                                            }
                                            Err(e) => {
                                                runtime_state.show_notification(format!(
                                                    "Failed to delete: {}",
                                                    e
                                                ));
                                            }
                                        }
                                    }
                                }
                                continue;
                            }
                            KeyCode::Esc => {
                                runtime_state.show_config_browser = false;
                                continue;
                            }
                            _ => continue,
                        }
                    }

                    // Handle palette editor input (intercept keys while editor is open)
                    if runtime_state.show_palette_editor {
                        use crossterm::event::{KeyCode, KeyModifiers};

                        if palette_editor_state.is_none() {
                            let current_palette = runtime_state.current_palette(&ALL_PALETTES);
                            palette_editor_state = Some(PaletteEditorState::new(&current_palette));
                        }

                        if let Some(ref mut state) = palette_editor_state {
                            match key_event.code {
                                KeyCode::Esc => {
                                    match state.mode {
                                        EditorMode::SaveDialog => {
                                            state.save_name_input.clear();
                                            state.mode = EditorMode::Editing;
                                        }
                                        EditorMode::LoadDialog => {
                                            state.mode = EditorMode::Editing;
                                        }
                                        EditorMode::Editing => {
                                            let original =
                                                Palette::Custom(state.original_colors.to_vec());
                                            renderer.set_palette(original);
                                            runtime_state.show_palette_editor = false;
                                            palette_editor_state = None;
                                        }
                                    }
                                    continue;
                                }
                                KeyCode::Left => {
                                    state.select_prev_color();
                                    continue;
                                }
                                KeyCode::Right => {
                                    state.select_next_color();
                                    continue;
                                }
                                KeyCode::Up => {
                                    if matches!(state.mode, EditorMode::LoadDialog) {
                                        if state.saved_palette_index > 0 {
                                            state.saved_palette_index -= 1;
                                        }
                                    } else {
                                        match state.selected_component {
                                            EditorComponent::Lightness => {
                                                state.adjust_lightness(0.02)
                                            }
                                            EditorComponent::Chroma => state.adjust_chroma(0.01),
                                            EditorComponent::Hue => state.adjust_hue(5.0),
                                        }
                                        renderer
                                            .set_palette(Palette::Custom(state.colors.to_vec()));
                                    }
                                    continue;
                                }
                                KeyCode::Down => {
                                    if matches!(state.mode, EditorMode::LoadDialog) {
                                        if state.saved_palette_index + 1
                                            < state.saved_palettes_list.len()
                                        {
                                            state.saved_palette_index += 1;
                                        }
                                    } else {
                                        match state.selected_component {
                                            EditorComponent::Lightness => {
                                                state.adjust_lightness(-0.02)
                                            }
                                            EditorComponent::Chroma => state.adjust_chroma(-0.01),
                                            EditorComponent::Hue => state.adjust_hue(-5.0),
                                        }
                                        renderer
                                            .set_palette(Palette::Custom(state.colors.to_vec()));
                                    }
                                    continue;
                                }
                                KeyCode::Char('h') | KeyCode::Char('H') => {
                                    state.selected_component = EditorComponent::Hue;
                                    continue;
                                }
                                KeyCode::Char('c') | KeyCode::Char('C') => {
                                    state.selected_component = EditorComponent::Chroma;
                                    continue;
                                }
                                KeyCode::Char('r') | KeyCode::Char('R') => {
                                    state.reset_to_original();
                                    renderer.set_palette(Palette::Custom(state.colors.to_vec()));
                                    continue;
                                }
                                KeyCode::Tab => {
                                    // Cycle OKLch component: L → C → H → L
                                    state.selected_component = state.selected_component.next();
                                    continue;
                                }
                                KeyCode::Char('s') | KeyCode::Char('S') => {
                                    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                                        state.mode = EditorMode::SaveDialog;
                                    }
                                    continue;
                                }
                                KeyCode::Char('l') => {
                                    // Lowercase l: select Lightness component
                                    state.selected_component = EditorComponent::Lightness;
                                    continue;
                                }
                                KeyCode::Char('L') => {
                                    // Uppercase L: open Load dialog
                                    if let Ok(palettes) = palette_manager::list_palettes() {
                                        state.saved_palettes_list = palettes;
                                        state.saved_palette_index = 0;
                                    }
                                    state.mode = EditorMode::LoadDialog;
                                    continue;
                                }
                                KeyCode::Enter => {
                                    match state.mode {
                                        EditorMode::Editing => {
                                            runtime_state.show_palette_editor = false;
                                            runtime_state.show_notification(
                                                "Custom palette applied".to_string(),
                                            );
                                            palette_editor_state = None;
                                        }
                                        EditorMode::SaveDialog => {
                                            if !state.save_name_input.is_empty() {
                                                let palette = palette_manager::SavedPalette::new(
                                                    state.save_name_input.clone(),
                                                    state.colors,
                                                );
                                                match palette_manager::save_palette(palette) {
                                                    Ok(_) => {
                                                        runtime_state.show_notification(format!(
                                                            "Palette '{}' saved",
                                                            state.save_name_input
                                                        ));
                                                        runtime_state.saved_palette_name =
                                                            Some(state.save_name_input.clone());
                                                    }
                                                    Err(e) => runtime_state.show_notification(
                                                        format!("Failed to save: {}", e),
                                                    ),
                                                }
                                            }
                                            state.save_name_input.clear();
                                            state.mode = EditorMode::Editing;
                                        }
                                        EditorMode::LoadDialog => {
                                            if let Some(palette) = state
                                                .saved_palettes_list
                                                .get(state.saved_palette_index)
                                            {
                                                state.colors = palette.to_rgb_colors();
                                                state.original_colors = palette.to_rgb_colors();
                                                state.base_palette_name = palette.name.clone();
                                                state.is_modified = false;
                                                runtime_state.saved_palette_name =
                                                    Some(palette.name.clone());
                                                runtime_state.show_notification(
                                                    "Palette loaded".to_string(),
                                                );
                                                renderer.set_palette(Palette::Custom(
                                                    state.colors.to_vec(),
                                                ));
                                            }
                                            state.mode = EditorMode::Editing;
                                        }
                                    }
                                    continue;
                                }
                                KeyCode::Char('\\')
                                | KeyCode::Char('|')
                                | KeyCode::Char('p')
                                | KeyCode::Char('P')
                                | KeyCode::Char('/') => {
                                    // Transition to dashboard overlay
                                    runtime_state.toggle_dashboard();
                                    continue;
                                }
                                KeyCode::Char(c)
                                    if !key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                                {
                                    if matches!(state.mode, EditorMode::SaveDialog) {
                                        if state.save_name_input.len() < 24 {
                                            state.save_name_input.push(c);
                                        }
                                        continue;
                                    }
                                }
                                KeyCode::Backspace => {
                                    if matches!(state.mode, EditorMode::SaveDialog) {
                                        state.save_name_input.pop();
                                        continue;
                                    }
                                }
                                _ => {}
                            }
                        }

                        if let Some(ref state) = palette_editor_state {
                            renderer.set_palette(Palette::Custom(state.colors.to_vec()));
                        }
                        continue;
                    }

                    if InputPoller::is_exit_key(&key_event) {
                        should_exit = true;
                        break;
                    }

                    let action = handle_key_event(&key_event);
                    match action {
                        ControlAction::Quit => {
                            should_exit = true;
                        }
                        ControlAction::TogglePause => {
                            runtime_state.toggle_pause();
                            runtime_state.pause_just_toggled = true;
                        }
                        ControlAction::Restart => {
                            sim.reset(
                                runtime_state.original_seed,
                                runtime_state.original_init_mode,
                            );
                        }
                        ControlAction::SetPreset(preset) => {
                            runtime_state.set_preset(preset);
                            let mut new_config = SimConfig::from(preset);
                            new_config.attractors = sim.config().attractors.clone();
                            new_config.attractor_strength = sim.config().attractor_strength;
                            new_config.food_image_path = sim.config().food_image_path.clone();
                            new_config.food_image_invert = sim.config().food_image_invert;
                            new_config.obstacles = sim.config().obstacles.clone();
                            new_config.obstacle_masks = sim.config().obstacle_masks.clone();
                            sim.update_config(new_config);
                            timer.set_time_scale(runtime_state.time_scale);
                        }
                        ControlAction::ComparePreset(preset) => {
                            runtime_state.toggle_preset_comparison(preset);
                        }
                        ControlAction::AdjustTimeScale(delta) => {
                            runtime_state.adjust_time_scale(delta);
                            timer.set_time_scale(runtime_state.time_scale);
                        }
                        ControlAction::CyclePalette => {
                            runtime_state.cycle_palette(num_palettes());
                            let new_palette = runtime_state.current_palette(&ALL_PALETTES);
                            renderer.set_palette(new_palette);
                        }
                        ControlAction::CyclePaletteReverse => {
                            runtime_state.cycle_palette_reverse(num_palettes());
                            let new_palette = runtime_state.current_palette(&ALL_PALETTES);
                            renderer.set_palette(new_palette);
                        }
                        ControlAction::CycleCharset => {
                            runtime_state.cycle_charset();
                            renderer.set_charset(runtime_state.current_charset());
                            runtime_state.show_notification(format!(
                                "Charset: {}",
                                charset_name(&runtime_state.current_charset())
                            ));
                        }
                        ControlAction::CycleCharsetReverse => {
                            runtime_state.cycle_charset_reverse();
                            renderer.set_charset(runtime_state.current_charset());
                            runtime_state.show_notification(format!(
                                "Charset: {}",
                                charset_name(&runtime_state.current_charset())
                            ));
                        }
                        ControlAction::ToggleDither => {
                            runtime_state.toggle_dither();
                            renderer.set_dither_mode(runtime_state.dither_mode);
                        }
                        ControlAction::CycleDitherMode => {
                            runtime_state.cycle_dither_mode();
                            renderer.set_dither_mode(runtime_state.dither_mode);
                        }
                        ControlAction::AdjustDitherIntensity(delta) => {
                            runtime_state.adjust_dither_intensity(delta);
                            renderer.set_dither_mode(runtime_state.dither_mode);
                        }
                        ControlAction::ToggleKeyboardHints => {
                            runtime_state.toggle_keyboard_hints();
                        }
                        ControlAction::ToggleControls => {
                            runtime_state.toggle_controls();
                        }
                        ControlAction::CloseOverlays => {
                            if runtime_state.any_overlay_open() {
                                runtime_state.close_all_overlays();
                            }
                            // If no overlays open, Esc does nothing (doesn't quit)
                        }
                        ControlAction::CycleOptionsCategory => {
                            const TOTAL_CATEGORIES: usize = 6;

                            if !runtime_state.show_controls {
                                runtime_state.close_all_overlays();
                                runtime_state.show_controls = true;
                            } else if runtime_state.controls_category_idx == TOTAL_CATEGORIES - 1 {
                                runtime_state.controls_category_idx = 0;
                            } else {
                                runtime_state.cycle_controls_category(true);
                            }
                        }
                        ControlAction::CycleOptionsCategoryReverse => {
                            const TOTAL_CATEGORIES: usize = 6;

                            if !runtime_state.show_controls {
                                runtime_state.close_all_overlays();
                                runtime_state.show_controls = true;
                            } else if runtime_state.controls_category_idx == 0 {
                                runtime_state.controls_category_idx = TOTAL_CATEGORIES - 1;
                            } else {
                                runtime_state.cycle_controls_category(false);
                            }
                        }
                        ControlAction::AdjustSensorAngle(delta) => {
                            let at_bound = runtime_state.adjust_sensor_angle(delta);
                            let mut new_config = sim.config().clone();
                            new_config.sensor_angle = runtime_state.sensor_angle;
                            sim.update_config(new_config);
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Sensor angle at {}°",
                                    runtime_state.sensor_angle
                                ));
                            }
                        }
                        ControlAction::AdjustSensorDistance(delta) => {
                            let at_bound = runtime_state.adjust_sensor_distance(delta);
                            let mut new_config = sim.config().clone();
                            new_config.sensor_distance = runtime_state.sensor_distance;
                            sim.update_config(new_config);
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Sensor distance at {:.1}",
                                    runtime_state.sensor_distance
                                ));
                            }
                        }
                        ControlAction::AdjustTurnAngle(delta) => {
                            let at_bound = runtime_state.adjust_rotation_angle(delta);
                            let mut new_config = sim.config().clone();
                            new_config.rotation_angle = runtime_state.rotation_angle;
                            sim.update_config(new_config);
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Turn angle at {}°",
                                    runtime_state.rotation_angle
                                ));
                            }
                        }
                        ControlAction::AdjustStepSize(delta) => {
                            let at_bound = runtime_state.adjust_step_size(delta);
                            let mut new_config = sim.config().clone();
                            new_config.step_size = runtime_state.step_size;
                            sim.update_config(new_config);
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Step size at {:.1}",
                                    runtime_state.step_size
                                ));
                            }
                        }
                        ControlAction::AdjustDecay(delta) => {
                            let at_bound = runtime_state.adjust_decay(delta);
                            let mut new_config = sim.config().clone();
                            new_config.decay_factor = runtime_state.decay_factor;
                            sim.update_config(new_config);
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Decay factor at {:.3}",
                                    runtime_state.decay_factor
                                ));
                            }
                        }
                        ControlAction::AdjustDeposit(delta) => {
                            let at_bound = runtime_state.adjust_deposit(delta);
                            let mut new_config = sim.config().clone();
                            new_config.deposit_amount = runtime_state.deposit_amount;
                            sim.update_config(new_config);
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Deposit amount at {:.1}",
                                    runtime_state.deposit_amount
                                ));
                            }
                        }
                        ControlAction::CycleDiffusionKernel => {
                            runtime_state.cycle_diffusion_kernel();
                            let mut new_config = sim.config().clone();
                            new_config.diffusion_kernel = runtime_state.diffusion_kernel;
                            sim.update_config(new_config);
                            runtime_state.show_notification(format!(
                                "Diffusion kernel: {}",
                                match runtime_state.diffusion_kernel {
                                    DiffusionKernel::Mean3x3 => "Mean3x3",
                                    DiffusionKernel::Gaussian => "Gaussian",
                                }
                            ));
                        }
                        ControlAction::AdjustDiffusionSigma(delta) => {
                            let at_bound = runtime_state.adjust_diffusion_sigma(delta);
                            let mut new_config = sim.config().clone();
                            new_config.diffusion_sigma = runtime_state.diffusion_sigma;
                            sim.update_config(new_config);
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Diffusion sigma at {:.2}",
                                    runtime_state.diffusion_sigma
                                ));
                            }
                        }
                        ControlAction::AdjustAttractorStrength(delta) => {
                            let at_bound = runtime_state.adjust_attractor_strength(delta);
                            let mut new_config = sim.config().clone();
                            new_config.attractor_strength = runtime_state.attractor_strength;
                            sim.update_config(new_config);
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Attractor strength at {:.1}",
                                    runtime_state.attractor_strength
                                ));
                            }
                        }
                        ControlAction::CycleMouseMode => {
                            runtime_state.cycle_mouse_mode();
                            runtime_state.show_notification(format!(
                                "Mouse mode: {}",
                                match runtime_state.mouse_mode {
                                    MouseInteractionMode::Disabled => "Disabled",
                                    MouseInteractionMode::Attract => "Attract",
                                    MouseInteractionMode::Repel => "Repel",
                                }
                            ));
                        }
                        ControlAction::CycleWindDirection => {
                            runtime_state.cycle_wind_direction();
                            let mut new_config = sim.config().clone();
                            new_config.wind = runtime_state.wind_direction.to_wind();
                            sim.update_config(new_config);
                            runtime_state.show_notification(format!(
                                "Wind: {}",
                                runtime_state.wind_direction.name()
                            ));
                        }
                        ControlAction::CycleWindDirectionReverse => {
                            runtime_state.cycle_wind_direction_reverse();
                            let mut new_config = sim.config().clone();
                            new_config.wind = runtime_state.wind_direction.to_wind();
                            sim.update_config(new_config);
                            runtime_state.show_notification(format!(
                                "Wind: {}",
                                runtime_state.wind_direction.name()
                            ));
                        }
                        ControlAction::AdjustTerrainStrength(delta) => {
                            let at_bound = runtime_state.adjust_terrain_strength(delta);
                            let mut new_config = sim.config().clone();
                            new_config.terrain_strength = runtime_state.terrain_strength;
                            sim.update_config(new_config);
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Terrain strength at {:.1}",
                                    runtime_state.terrain_strength
                                ));
                            }
                        }
                        ControlAction::CycleTerrainType => {
                            runtime_state.cycle_terrain_type();
                            let mut new_config = sim.config().clone();
                            new_config.terrain = runtime_state.terrain_type;
                            sim.update_config(new_config);
                            runtime_state.show_notification(format!(
                                "Terrain: {}",
                                match runtime_state.terrain_type {
                                    TerrainType::None => "None",
                                    TerrainType::Smooth => "Smooth",
                                    TerrainType::Turbulent => "Turbulent",
                                    TerrainType::Mixed => "Mixed",
                                }
                            ));
                        }
                        ControlAction::ToggleAutoNormalize => {
                            runtime_state.toggle_auto_normalize();
                            current_auto_normalize = runtime_state.auto_normalize;
                            adaptive_brightness = AdaptiveBrightness::new(
                                args.normalize_window,
                                current_auto_normalize,
                            );
                        }
                        ControlAction::CycleMotionBlur => {
                            runtime_state.cycle_motion_blur();
                            runtime_state.show_notification(format!(
                                "Motion blur: {} frames",
                                runtime_state.motion_blur_frames
                            ));
                        }
                        ControlAction::AdjustMaxBrightness(delta) => {
                            let at_bound = runtime_state.adjust_max_brightness(delta);
                            _current_max_brightness = runtime_state.max_brightness;
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Max brightness at {:.1}",
                                    runtime_state.max_brightness
                                ));
                            }
                        }
                        ControlAction::SaveFrameToPng => {
                            use crate::export::png::save_frame_as_png;

                            match save_frame_as_png(
                                downsampled.cells(),
                                term_width as usize,
                                term_height as usize,
                                current_palette.clone(),
                                runtime_state.reverse_palette,
                                runtime_state.invert_palette,
                                hue_offset,
                                max_brightness,
                            ) {
                                Ok(filename) => {
                                    runtime_state
                                        .show_notification(format!("Frame saved: {}", filename));
                                }
                                Err(e) => {
                                    runtime_state.show_notification(format!("Error: {}", e));
                                }
                            }
                        }
                        ControlAction::ToggleFastMode => {
                            runtime_state.toggle_fast_mode();
                            runtime_state.show_notification(format!(
                                "Fast mode: {}",
                                if runtime_state.fast_mode_enabled {
                                    "On"
                                } else {
                                    "Off"
                                }
                            ));
                        }
                        ControlAction::CyclePaletteShiftSpeed => {
                            runtime_state.cycle_palette_shift_speed();
                            runtime_state.show_notification(format!(
                                "Palette shift: {}",
                                match runtime_state.palette_shift_speed {
                                    PaletteShiftSpeed::Off => "Off",
                                    PaletteShiftSpeed::Slow => "Slow (5°/s)",
                                    PaletteShiftSpeed::Medium => "Medium (15°/s)",
                                    PaletteShiftSpeed::Fast => "Fast (45°/s)",
                                }
                            ));
                        }
                        ControlAction::ToggleInvertPalette => {
                            runtime_state.toggle_invert_palette();
                            renderer.set_invert_palette(runtime_state.invert_palette);
                        }
                        ControlAction::ToggleReversePalette => {
                            runtime_state.toggle_reverse_palette();
                            renderer.set_reverse_palette(runtime_state.reverse_palette);
                        }
                        ControlAction::CycleIntensityMapping => {
                            runtime_state.cycle_intensity_mapping(false);
                            runtime_state.show_notification(format!(
                                "Intensity: {}",
                                runtime_state.intensity_mapping_name()
                            ));
                            renderer.set_intensity_mapping(Some(
                                runtime_state.intensity_mapping.clone(),
                            ));
                        }
                        ControlAction::CycleIntensityMappingReverse => {
                            runtime_state.cycle_intensity_mapping(true);
                            runtime_state.show_notification(format!(
                                "Intensity: {}",
                                runtime_state.intensity_mapping_name()
                            ));
                            renderer.set_intensity_mapping(Some(
                                runtime_state.intensity_mapping.clone(),
                            ));
                        }
                        ControlAction::ResetToDefaults => {
                            runtime_state.reset_to_defaults();
                            let new_config = SimConfig::from(runtime_state.current_preset);
                            sim.update_config(new_config);
                            timer.set_time_scale(runtime_state.time_scale);
                            _current_max_brightness = runtime_state.max_brightness;
                            renderer.set_invert_palette(runtime_state.invert_palette);
                            renderer.set_reverse_palette(runtime_state.reverse_palette);
                            hue_offset = 0.0;
                            let notification = if runtime_state.cli_overrides.is_some() {
                                "Reset to CLI parameters"
                            } else {
                                "Reset to defaults"
                            };
                            runtime_state.show_notification(notification.to_string());
                        }
                        ControlAction::ToggleDashboard => {
                            runtime_state.toggle_dashboard();
                        }
                        ControlAction::SetIntensityMapping(_) => {}
                        ControlAction::ShowConfigBrowser => {
                            runtime_state.close_all_overlays();
                            runtime_state.show_config_browser = true;
                            runtime_state.config_browser_selected_index = 0;
                        }
                        ControlAction::ShowConfigSaveDialog => {
                            runtime_state.close_all_overlays();
                            runtime_state.show_config_save_dialog = true;
                            runtime_state.config_save_name_input.clear();
                        }
                        ControlAction::RandomizeParams => {
                            runtime_state.randomize_params();
                            apply_random_config(
                                &runtime_state,
                                sim,
                                &mut renderer,
                                &ALL_PALETTES,
                                &mut _current_max_brightness,
                            );

                            runtime_state.show_notification("Parameters Randomized!".to_string());
                        }
                        ControlAction::Undo => {
                            if runtime_state.undo().is_some() {
                                // Apply the undone state to simulation
                                let mut new_config = sim.config().clone();
                                new_config.sensor_angle = runtime_state.sensor_angle;
                                new_config.rotation_angle = runtime_state.rotation_angle;
                                new_config.step_size = runtime_state.step_size;
                                new_config.decay_factor = runtime_state.decay_factor;
                                new_config.deposit_amount = runtime_state.deposit_amount;
                                new_config.diffusion_kernel = runtime_state.diffusion_kernel;
                                new_config.diffusion_sigma = runtime_state.diffusion_sigma;
                                new_config.max_brightness = runtime_state.max_brightness;
                                new_config.terrain = runtime_state.terrain_type;
                                new_config.terrain_strength = runtime_state.terrain_strength;
                                sim.update_config(new_config);

                                renderer.set_palette(runtime_state.current_palette(&ALL_PALETTES));
                                renderer.set_invert_palette(runtime_state.invert_palette);
                                renderer.set_reverse_palette(runtime_state.reverse_palette);
                                renderer.set_dither_mode(runtime_state.dither_mode);

                                runtime_state.show_notification("Undo successful".to_string());
                            } else {
                                runtime_state.show_notification("Nothing to undo".to_string());
                            }
                        }
                        ControlAction::Redo => {
                            if runtime_state.redo().is_some() {
                                // Apply the redone state to simulation
                                let mut new_config = sim.config().clone();
                                new_config.sensor_angle = runtime_state.sensor_angle;
                                new_config.rotation_angle = runtime_state.rotation_angle;
                                new_config.step_size = runtime_state.step_size;
                                new_config.decay_factor = runtime_state.decay_factor;
                                new_config.deposit_amount = runtime_state.deposit_amount;
                                new_config.diffusion_kernel = runtime_state.diffusion_kernel;
                                new_config.diffusion_sigma = runtime_state.diffusion_sigma;
                                new_config.max_brightness = runtime_state.max_brightness;
                                new_config.terrain = runtime_state.terrain_type;
                                new_config.terrain_strength = runtime_state.terrain_strength;
                                sim.update_config(new_config);

                                renderer.set_palette(runtime_state.current_palette(&ALL_PALETTES));
                                renderer.set_invert_palette(runtime_state.invert_palette);
                                renderer.set_reverse_palette(runtime_state.reverse_palette);
                                renderer.set_dither_mode(runtime_state.dither_mode);

                                runtime_state.show_notification("Redo successful".to_string());
                            } else {
                                runtime_state.show_notification("Nothing to redo".to_string());
                            }
                        }
                        ControlAction::CycleTheme => {
                            runtime_state.cycle_theme();
                            runtime_state.show_notification(format!(
                                "Theme: {}",
                                runtime_state.current_theme_name()
                            ));
                        }
                        ControlAction::CycleThemeReverse => {
                            runtime_state.cycle_theme_reverse();
                            runtime_state.show_notification(format!(
                                "Theme: {}",
                                runtime_state.current_theme_name()
                            ));
                        }
                        ControlAction::ShowPaletteEditor => {
                            runtime_state.toggle_palette_editor();
                            if runtime_state.show_palette_editor {
                                let current_palette = runtime_state.current_palette(&ALL_PALETTES);
                                palette_editor_state =
                                    Some(PaletteEditorState::new(&current_palette));
                            } else {
                                palette_editor_state = None;
                            }
                        }
                        ControlAction::ToggleTrailAge => {
                            runtime_state.trail_age_enabled = !runtime_state.trail_age_enabled;
                            sim.set_compute_trail_age(runtime_state.trail_age_enabled);
                            runtime_state.show_notification(format!(
                                "Trail Age: {}",
                                if runtime_state.trail_age_enabled {
                                    "On"
                                } else {
                                    "Off"
                                }
                            ));
                        }
                        ControlAction::ToggleTrailDelta => {
                            runtime_state.trail_delta_enabled = !runtime_state.trail_delta_enabled;
                            sim.set_compute_trail_delta(runtime_state.trail_delta_enabled);
                            runtime_state.show_notification(format!(
                                "Trail Delta: {}",
                                if runtime_state.trail_delta_enabled {
                                    "On"
                                } else {
                                    "Off"
                                }
                            ));
                        }
                        ControlAction::ToggleGradientMagnitude => {
                            runtime_state.gradient_magnitude_enabled =
                                !runtime_state.gradient_magnitude_enabled;
                            sim.set_compute_gradient_magnitude(
                                runtime_state.gradient_magnitude_enabled,
                            );
                            runtime_state.show_notification(format!(
                                "Edge Glow: {}",
                                if runtime_state.gradient_magnitude_enabled {
                                    "On"
                                } else {
                                    "Off"
                                }
                            ));
                        }
                        ControlAction::None => {}
                    }
                }
                Event::Mouse(mouse_event) => {
                    if runtime_state.mouse_mode == MouseInteractionMode::Disabled {
                        continue;
                    }

                    let event_type =
                        if matches!(mouse_event.kind, crossterm::event::MouseEventKind::Down(_)) {
                            MouseEventType::Down
                        } else if matches!(
                            mouse_event.kind,
                            crossterm::event::MouseEventKind::Drag(_)
                        ) {
                            MouseEventType::Drag
                        } else if matches!(
                            mouse_event.kind,
                            crossterm::event::MouseEventKind::Moved
                        ) {
                            MouseEventType::Moved
                        } else {
                            continue;
                        };

                    if term_width == 0 || term_height == 0 {
                        continue;
                    }

                    let term_x = mouse_event.column as usize - 1;
                    let term_y = mouse_event.row as usize - 1;

                    let sim_x = (term_x as f32 / term_width as f32) * sim.width() as f32;
                    let sim_y = (term_y as f32 / term_height as f32) * sim.height() as f32;

                    let strength = match runtime_state.mouse_mode {
                        MouseInteractionMode::Attract => 2.0,
                        MouseInteractionMode::Repel => -2.0,
                        MouseInteractionMode::Disabled => 0.0,
                    };

                    match event_type {
                        MouseEventType::Down => {
                            sim.add_mouse_attractor(sim_x, sim_y, strength);
                            if args.verbose {
                                eprintln!(
                                    "[Mouse] {} at ({:.0}, {:.0})",
                                    match runtime_state.mouse_mode {
                                        MouseInteractionMode::Attract => "Attractor",
                                        MouseInteractionMode::Repel => "Repeller",
                                        _ => "Effect",
                                    },
                                    sim_x,
                                    sim_y
                                );
                            }
                        }
                        MouseEventType::Drag => {
                            sim.add_mouse_attractor(sim_x, sim_y, strength);
                            if args.verbose {
                                eprintln!(
                                    "[Mouse] Drag {} at ({:.0}, {:.0})",
                                    match runtime_state.mouse_mode {
                                        MouseInteractionMode::Attract => "attract",
                                        MouseInteractionMode::Repel => "repel",
                                        _ => "effect",
                                    },
                                    sim_x,
                                    sim_y
                                );
                            }
                        }
                        MouseEventType::Moved => {
                            if args.verbose {
                                eprintln!("[Mouse] Move at ({:.0}, {:.0})", sim_x, sim_y);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // If pause was toggled during input, re-render immediately with the new pause state
        // This gives instant visual feedback instead of waiting for the next frame
        if runtime_state.pause_just_toggled {
            runtime_state.pause_just_toggled = false;

            // Rebuild pause overlays with new state
            let (pause_logo_overlay, pause_logo_x, pause_logo_y) = if runtime_state.is_paused {
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

                let brightness_map = if runtime_state
                    .pause_logo_cache
                    .as_ref()
                    .is_some_and(|(cw, _, _)| *cw == logo_w)
                {
                    runtime_state.pause_logo_cache.as_ref().unwrap().2.clone()
                } else {
                    let map = load_logo_from_memory(FOOD_IMAGE_PNG, pixel_w, pixel_h, true)
                        .unwrap_or_else(|_| vec![0.0; pixel_w * pixel_h]);
                    runtime_state.pause_logo_cache = Some((logo_w, pixel_h, map.clone()));
                    map
                };

                let current_palette = ALL_PALETTES[runtime_state.palette_index].clone();
                let logo_mapping = args
                    .logo_mapping()
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| runtime_state.intensity_mapping.clone());
                let logo = PauseOverlay::build_logo(
                    &brightness_map,
                    logo_w,
                    logo_h,
                    current_palette,
                    runtime_state.reverse_palette,
                    runtime_state.invert_palette,
                    0.0,
                    Some(&logo_mapping),
                );

                runtime_state.pause_frame_counter += 1;

                let actual_logo_h = logo.lines.len();
                let lx = (term_width as usize).saturating_sub(logo_w) / 2;
                let drawable_h = (term_height as usize).saturating_sub(1);
                let ly = drawable_h.saturating_sub(actual_logo_h) / 2;

                (Some(logo), lx, ly)
            } else {
                runtime_state.pause_frame_counter = 0;
                (None, 0, 0)
            };

            // Build status line
            let diffusion_kernel_name = match current_config.diffusion_kernel {
                DiffusionKernel::Mean3x3 => "Mean3x3",
                DiffusionKernel::Gaussian => "Gaussian",
            };
            let (status_line, status_colors) = OverlayRenderer::build_status_line(
                runtime_state.is_paused,
                runtime_state.current_preset,
                runtime_state.time_scale,
                current_palette.clone(),
                runtime_state.dither_mode,
                term_width as usize,
                Some(sim.agent_count()),
                Some(diffusion_kernel_name),
                !runtime_state.undo_stack.is_empty(),
                !runtime_state.redo_stack.is_empty(),
                Some(ui_accent),
            );
            let status_x = OverlayRenderer::status_line_x(&status_line, term_width as usize);
            let status_data = Some((status_line, status_x, status_colors));

            // Re-render with updated pause state
            if args.species_colors && sim.config().separate_species_trails {
                let species_trail_maps = sim.trail_maps_for_species_colors();
                let species_rgb_colors = extract_species_rgb_colors(&current_config);
                let combined: Vec<_> = species_trail_maps
                    .iter()
                    .zip(species_rgb_colors.iter())
                    .map(|(tm, color)| (*tm, *color))
                    .collect();
                renderer.render_multi_species_with_overlay(
                    &combined,
                    sim.width(),
                    sim.height(),
                    max_brightness.max(1.0),
                    if runtime_state.is_paused {
                        Some(runtime_state.pause_frame_counter)
                    } else {
                        None
                    },
                    pause_logo_overlay
                        .as_ref()
                        .map(|v| (v, pause_logo_x, pause_logo_y)),
                    None,
                    None,
                    status_data,
                    None,
                    None,
                    grid_renderer.as_ref(),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(&runtime_state.panel_style),
                    runtime_state.focused_overlay,
                )?;
            } else {
                renderer.render_with_overlay(
                    downsampled.cells(),
                    max_brightness.max(1.0),
                    if runtime_state.is_paused {
                        Some(runtime_state.pause_frame_counter)
                    } else {
                        None
                    },
                    pause_logo_overlay
                        .as_ref()
                        .map(|v| (v, pause_logo_x, pause_logo_y)),
                    None,
                    None,
                    status_data,
                    None,
                    None,
                    grid_renderer.as_ref(),
                    None,
                    None,
                    None,
                    None,
                    None,
                    Some(&runtime_state.panel_style),
                    runtime_state.focused_overlay,
                )?;
            }
        }

        if args.verbose {
            eprintln!(
                "FPS: {:.1} (avg: {:.1}) | Sim: {:.1}ms | Render: {:.1}ms | Frame: {}",
                timer.current_fps(),
                timer.average_fps(),
                timer.sim_duration().as_secs_f64() * 1000.0,
                timer.render_duration().as_secs_f64() * 1000.0,
                timer.frame_count(),
            );
        }

        if should_exit {
            break;
        }

        if timer.should_adjust_fps() {
            if let Some(new_fps) = timer.get_adjusted_fps() {
                timer.apply_fps_adjustment(new_fps);
                runtime_state
                    .show_notification(format!("Adaptive FPS: {} -> {}", args.fps, new_fps));
            }
        }

        if timer.fps_adjusted_notification {
            timer.fps_adjusted_notification = false;
        }

        runtime_state.update_notifications();
        timer.tick();
    }

    if runtime_state.mouse_mode != MouseInteractionMode::Disabled {
        let _ = crate::terminal::disable_mouse_tracking();
    }

    Ok(())
}

/// Gets terminal size from environment variables or crossterm.
pub fn get_terminal_size() -> (usize, usize) {
    let width = std::env::var("COLUMNS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok());
    let height = std::env::var("LINES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok());

    if let (Some(w), Some(h)) = (width, height) {
        return (w, h);
    }

    match crossterm::terminal::size() {
        Ok((w, h)) => (w as usize, h as usize),
        Err(_) => (80, 24),
    }
}
