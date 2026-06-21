//! Interactive simulation runner and main event loop.
//!
//! This module contains the main simulation runner that handles the interactive
//! terminal-based simulation with real-time user input, rendering, and overlay management.

use std::io::{self, Write};

use clap::Parser as _;

use crate::app::{
    apply_live_params, apply_random_config, extract_species_rgb_colors, REFERENCE_TIME_STEP,
};
use crate::cli::{self, Args, ColorMode, Mode, Palette};
use crate::config_defaults::warmup::{TRANSITION_DURATION_FRAMES, WARMUP_SPEED_MULTIPLIER};
use crate::config_manager;
use crate::export::GifExporter;
use crate::export::WebmExporter;
use crate::food_image::FOOD_IMAGE_PNG;
use crate::overlay::{OverlayInputManager, OverlayInputResult, OverlayType};
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
use crate::terminal::renderer::{ChromeSnapshot, TerminalRenderer};
use crate::terminal::screen::TerminalScreen;
use crate::terminal::signal::is_shutdown_requested;
use crate::terminal::timing::FrameTimer;
use crossterm::event::Event;
use memory_stats::memory_stats;

/// Target frame time at 30 FPS, in milliseconds.
const TARGET_FRAME_TIME_MS: f32 = 33.333;

/// Updates food persistence attractors with fade-out effect.
///
/// Gradually reduces the strength of food attractors over time using
/// quadratic easing for a smooth fade-out effect.
fn update_food_persistence(sim: &mut Simulation, runtime_state: &mut RuntimeState) {
    let duration = runtime_state.app.food_persist_duration;
    if !runtime_state.food_persist_enabled || runtime_state.is_paused || duration == 0 {
        return;
    }

    runtime_state.food_persist_counter += 1;

    if runtime_state.food_persist_counter <= duration {
        let progress = runtime_state.food_persist_counter as f32 / duration as f32;
        let fade_factor: f32 = (1.0 - progress).powi(2); // Quadratic fade-out

        // Update attractor strengths without cloning entire config
        let attractors: Vec<Attractor> = runtime_state
            .initial_food_attractors
            .iter()
            .map(|a| Attractor::new(a.x, a.y, a.strength * fade_factor))
            .collect();
        sim.update_attractors(attractors);
    } else if runtime_state.food_persist_counter == duration + 1 {
        // Remove all food attractors when duration expires
        sim.update_attractors(Vec::new());
    }
}

/// Checks if simulation should auto-reset based on entropy collapse.
///
/// Monitors entropy levels and resets the simulation if it collapses
/// (entropy stays below threshold for specified duration).
fn check_auto_reset(sim: &mut Simulation, runtime_state: &mut RuntimeState, entropy: f32) {
    if !runtime_state.app.auto_reset || runtime_state.is_paused {
        return;
    }

    let should_reset = runtime_state.track_entropy(
        entropy,
        runtime_state.app.auto_reset_entropy_threshold,
        runtime_state.app.auto_reset_duration_frames,
    );

    if should_reset {
        let new_seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Use the live original init mode (the apply seam updates it on restart),
        // NOT the startup `init_mode` local which is stale after a config load.
        let init_mode = runtime_state.original_init_mode;
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

/// Returns `true` when a key event should immediately exit the application
/// in screensaver mode.
///
/// Any key press exits; key release/repeat events (emitted on some platforms,
/// e.g. Windows) are ignored so a single keystroke does not double-fire.
fn screensaver_exit_on_key(mode: &Mode, key_event: &crossterm::event::KeyEvent) -> bool {
    *mode == Mode::Screensaver && key_event.kind == crossterm::event::KeyEventKind::Press
}

/// Runs the interactive simulation loop (Live or Screensaver mode).
///
/// Handles terminal setup, input processing, simulation updates, and rendering
/// loop.
#[allow(unused_assignments)]
pub fn run_simulation(
    sim: &mut Simulation,
    args: &Args,
    mode: Mode,
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

    let startup_profile = crate::profile::Profile::resolve_from_args(args)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let config = startup_profile.sim.clone();
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
    // Dither is dev-only for v0.1.0: runtime keys work only when it was
    // explicitly enabled at startup via the (hidden) CLI flags.
    let dither_unlocked = !matches!(dither_mode, DitherMode::None);
    renderer.set_ascii_contrast(args.ascii_contrast);
    renderer.set_window_frame(config.window_frame);
    let mut timer = FrameTimer::with_time_scale(args.fps, args.frame_delay, args.time_scale);
    timer.set_adaptive_fps(args.auto_fps);
    let input_poller = InputPoller::new();

    let (mut term_width, mut term_height) = screen.get_size()?;
    renderer.set_dimensions(term_width as usize, term_height as usize);

    // Compute initial window layout for windowed (non-fullscreen) chrome styles.
    // `mut` because the config-load apply seam recomputes it on load.
    let mut window = crate::render::window::Window {
        aspect: config.aspect,
        padding: config.window_padding,
        min_sim_size: config.min_sim_size,
        min_frame_size: config.min_frame_size,
    };
    {
        use crate::simulation::config::ChromeStyle;
        let initial_layout = if matches!(config.chrome_style, ChromeStyle::Fullscreen) {
            None
        } else {
            let l = window.compute_rects(term_width as usize, term_height as usize);
            if matches!(l.fallback, crate::render::window::FallbackMode::Fullscreen) {
                None
            } else {
                Some(l)
            }
        };
        renderer.set_window_layout(initial_layout);
    }

    let seed = args.seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });

    let initial_preset = args.preset.unwrap_or(Preset::Organic);

    let resolved = startup_profile.render.clone();

    let mut runtime_state = RuntimeState::new(
        seed,
        init_mode,
        initial_preset,
        mouse_mode,
        args.mouse_timeout,
        &config,
        args.pause_style,
        args.pause_logo,
        args.pause_pulse_draw_mode,
    );
    // Classify the startup invocation: bare `--preset <p>` (no other overrides, no seed pin)
    // → Preset(p); anything more complex (extra flags, seed, no preset) → StartupCli.
    let startup_ov = crate::profile_overrides::ProfileOverrides::from_args(args)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    runtime_state.active_source = if let Some(p) = args.preset {
        // Build a clap-parsed template for bare `--preset <name>` to get correct clap
        // defaults (not Rust struct defaults which differ on fps, time_scale, etc.).
        let preset_name = crate::terminal::control::preset_name(p);
        let template_args = Args::parse_from(["tslime", "--preset", preset_name]);
        let template = crate::profile_overrides::ProfileOverrides::from_args(&template_args)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
        match startup_ov.bare_preset_against(&template) {
            Some(preset) => crate::profile::ProfileSource::Preset(preset),
            None => crate::profile::ProfileSource::StartupCli,
        }
    } else {
        crate::profile::ProfileSource::StartupCli
    };
    // Seed the live app-runtime config + active overrides from the startup profile so
    // the runner reads warmup/auto-reset/grid/food from rs.app (the single live source).
    runtime_state.app = startup_profile.app.clone();
    runtime_state.active_overrides = startup_ov;
    runtime_state.preload_pause_logo(term_width as usize, term_height as usize);
    runtime_state.dither_mode = dither_mode;
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
    // decay_gamma / diffuse_weight come from the assembled config via
    // RuntimeState::new — do not re-clobber them from raw CLI args here.
    if args.stats {
        runtime_state.overlay_state.open(OverlayType::Dashboard);
    }
    if args.trail_age {
        sim.set_compute_trail_age(true);
    }
    if args.trail_delta {
        sim.set_compute_trail_delta(true);
    }
    if args.gradient_magnitude {
        sim.set_compute_gradient_magnitude(true);
    }
    crate::app::apply_render_config(&resolved, &mut runtime_state, &mut renderer, sim);
    runtime_state.set_render_baseline(resolved.clone());

    // Initialize food persistence
    if args.food_persist && init_mode == InitMode::Food {
        runtime_state.food_persist_enabled = true;
        runtime_state.initial_food_attractors = Simulation::create_food_attractors(
            args.resolution.width,
            args.resolution.height,
            &args.food,
            args.food_invert,
            args.food_scale,
            runtime_state.app.food_persist_strength,
            0.3,
        );

        let mut new_config = sim.config().clone();
        new_config
            .attractors
            .extend(runtime_state.initial_food_attractors.clone());
        sim.update_config(new_config);
    }

    if args.species_colors_enabled() {
        let species_rgb_colors = extract_species_rgb_colors(&config);
        renderer.set_species_colors(true, species_rgb_colors);
    }

    let mut adaptive_brightness =
        AdaptiveBrightness::new(args.normalize_window, args.auto_normalize);
    let mut hue_offset: f32 = 0.0;

    let mut current_auto_normalize = args.auto_normalize;

    if args.random {
        runtime_state.randomize_params();
        apply_random_config(&runtime_state, sim, &mut renderer, &ALL_PALETTES);
    }

    let start_time = std::time::Instant::now();

    let mut grid_renderer = crate::app::build_grid_renderer(
        &runtime_state.app,
        (term_width as usize, term_height as usize),
    );

    // Compute sim render dimensions (may be smaller than terminal in windowed mode)
    let compute_render_dims = |tw: usize, th: usize| -> (usize, usize) {
        use crate::simulation::config::ChromeStyle;
        if matches!(config.chrome_style, ChromeStyle::Fullscreen) {
            return (tw, th);
        }
        let l = window.compute_rects(tw, th);
        if matches!(l.fallback, crate::render::window::FallbackMode::Fullscreen) {
            (tw, th)
        } else {
            (l.sim_w, l.sim_h)
        }
    };
    let (initial_render_w, initial_render_h) =
        compute_render_dims(term_width as usize, term_height as usize);

    // Pre-allocate frame buffers to avoid per-frame allocations
    let mut downsampled_frame =
        crate::render::downsample::DownsampledFrame::new(initial_render_w, initial_render_h);
    let mut aux_frame = crate::render::downsample::AuxFrame {
        width: initial_render_w,
        height: initial_render_h,
        cells: vec![
            crate::render::downsample::AuxCell::default();
            initial_render_w * initial_render_h
        ],
    };
    let mut blended_trail_buffer: Vec<f32> = Vec::new();

    #[cfg(feature = "audio")]
    let mut choir: Option<crate::audio::Choir> = if args.choir {
        match crate::audio::Choir::try_new(args.choir_volume.clamp(0.0, 1.0)) {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("Choir mode disabled: {e}");
                None
            }
        }
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
                // Recompute window layout on resize; derive render dims from the same layout
                // to avoid calling compute_rects a second time.
                let (new_render_w, new_render_h) = {
                    use crate::simulation::config::ChromeStyle;
                    let new_layout = if matches!(config.chrome_style, ChromeStyle::Fullscreen) {
                        None
                    } else {
                        let l = window.compute_rects(term_width as usize, term_height as usize);
                        if matches!(l.fallback, crate::render::window::FallbackMode::Fullscreen) {
                            None
                        } else {
                            Some(l)
                        }
                    };
                    // Derive render dims from the computed layout before moving it into renderer
                    let dims = new_layout
                        .as_ref()
                        .map(|l| (l.sim_w, l.sim_h))
                        .unwrap_or((term_width as usize, term_height as usize));
                    renderer.set_window_layout(new_layout);
                    dims
                };
                // Reinitialize grid with new dimensions
                if let Some(grid) = &mut grid_renderer {
                    grid.initialize(term_width as usize, term_height as usize);
                }
                // Resize frame buffers to sim render dimensions (may differ in windowed mode)
                downsampled_frame =
                    crate::render::downsample::DownsampledFrame::new(new_render_w, new_render_h);
                aux_frame.width = new_render_w;
                aux_frame.height = new_render_h;
                aux_frame.cells = vec![
                    crate::render::downsample::AuxCell::default();
                    new_render_w * new_render_h
                ];
            }
        }

        if term_width == 0 || term_height == 0 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            continue;
        }

        // Per-frame elapsed time, scaled by time_scale (NOT raw wall clock). Used
        // for UI effects (hue shift, chrome fade via dt_wall below) — never to step
        // the simulation, which uses the fixed timestep sim_dt.
        let dt = timer.delta_time();

        // Clamp dt to avoid UI animation jumps during lag spikes (max 0.1s / 10 FPS)
        let dt = dt.min(0.1);

        // Fixed simulation timestep, decoupled from frame-write jitter. A blocked
        // write (e.g. terminal back-pressure while holding a key) inflates wall `dt`
        // and previously made the sim lurch forward one big step, which read as
        // flicker. Stepping by a fixed amount keeps motion smooth regardless of I/O.
        let sim_dt = timer.fixed_delta();

        // Chrome fade-out (500ms collapse) is UI, not simulation — divide the
        // time_scale back out so it animates at wall-clock speed.
        let dt_wall = dt / runtime_state.time_scale.max(f32::EPSILON);
        runtime_state.advance_fade(dt_wall);

        let in_warmup = !runtime_state.app.skip_warmup
            && runtime_state.is_in_warmup(runtime_state.app.warmup_frames);

        let frames_since_warmup = runtime_state
            .warmup_counter
            .saturating_sub(runtime_state.app.warmup_frames);
        let in_transition = frames_since_warmup < TRANSITION_DURATION_FRAMES;

        // Warmup→normal handoff: 0.0 during warmup, ramps 0→1 over
        // TRANSITION_DURATION_FRAMES, then holds at 1.0.
        let fade_factor = if in_warmup {
            0.0
        } else if in_transition {
            frames_since_warmup as f32 / TRANSITION_DURATION_FRAMES as f32
        } else {
            1.0
        };

        if !runtime_state.is_paused {
            timer.start_sim();

            // Ramp sim speed from WARMUP_SPEED_MULTIPLIER up to 1.0 as warmup ends.
            let speed_multiplier =
                WARMUP_SPEED_MULTIPLIER + (1.0 - WARMUP_SPEED_MULTIPLIER) * fade_factor;

            // Clamp against floating-point drift.
            let speed_multiplier = speed_multiplier.clamp(WARMUP_SPEED_MULTIPLIER, 1.0);

            let adjusted_dt = sim_dt * speed_multiplier;
            sim.update(adjusted_dt / REFERENCE_TIME_STEP);

            // Cap the counter so it stops once warmup + transition are complete.
            if !runtime_state.app.skip_warmup
                && runtime_state.warmup_counter
                    < runtime_state.app.warmup_frames + TRANSITION_DURATION_FRAMES
            {
                runtime_state.increment_warmup();
            }

            timer.end_sim_start_render();
        } else {
            timer.start_sim();
            timer.end_sim_start_render();
        }

        #[cfg(feature = "audio")]
        if let Some(ref c) = choir {
            if runtime_state.is_paused {
                c.silence_all();
            } else {
                let tm = sim.trail_map();
                crate::audio::update_voices_from_trail(
                    c,
                    tm.current(),
                    tm.width(),
                    tm.height(),
                    runtime_state.max_brightness.max(1.0),
                );
            }
        }

        let sim_width = sim.width();
        let sim_height = sim.height();
        let sim_dims = sim_width * sim_height;
        let agent_count = sim.agent_count();

        // Get blended trail first (takes &mut self)
        sim.trail_map_blended(&mut blended_trail_buffer);
        crate::app::fold_afterglow(
            &mut blended_trail_buffer,
            sim.afterglow_lag(),
            runtime_state.afterglow,
        );
        // Use sim render dimensions from downsampled_frame (may differ from terminal in windowed mode)
        let render_w = downsampled_frame.width();
        let render_h = downsampled_frame.height();
        downsample(
            &blended_trail_buffer,
            sim_width,
            sim_height,
            render_w,
            render_h,
            &mut downsampled_frame,
        );

        // Compute auxiliary frame for trail age / temporal delta / gradient / temporal color
        let current_aux_frame = if runtime_state.trail_age_enabled
            || runtime_state.trail_delta_enabled
            || runtime_state.gradient_magnitude_enabled
            || runtime_state.temporal_color > 0.0
        {
            let trail_age = if runtime_state.trail_age_enabled {
                sim.trail_age()
            } else {
                None
            };
            let trail_delta = if runtime_state.trail_delta_enabled {
                sim.trail_delta()
            } else {
                None
            };
            let gradient_mag = if runtime_state.gradient_magnitude_enabled {
                sim.gradient_magnitude()
            } else {
                None
            };

            crate::render::downsample::downsample_aux(
                trail_age,
                trail_delta,
                gradient_mag,
                sim.temporal_diff(),
                sim_width,
                sim_height,
                render_w,
                render_h,
                &mut aux_frame,
            );
            Some(&aux_frame)
        } else {
            None
        };
        renderer.set_visual_fx(
            current_aux_frame.cloned(),
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
        renderer.set_temporal(
            runtime_state.temporal_color,
            runtime_state.temporal_mode,
            runtime_state.temporal_accent,
        );

        // Narrow live reads instead of cloning the whole SimConfig (which carries heap Vecs
        // including obstacle masks). DiffusionKernel is Copy; species colors are the only Vec
        // consumers, collected once here and reused at both render sites.
        let current_diffusion_kernel = sim.config().diffusion_kernel;
        let current_species_rgb: Vec<crate::render::palette::RgbColor> =
            extract_species_rgb_colors(sim.config());

        adaptive_brightness.update(downsampled_frame.cells());
        let mut max_brightness = if current_auto_normalize {
            adaptive_brightness.get_max_brightness()
        } else {
            runtime_state.max_brightness
        };

        // Warmup brightness boost, eased out with the same fade_factor as the
        // speed ramp.
        if in_warmup || in_transition {
            // Inverse of fade_factor: 1.0 during warmup, easing to 0.0 as the
            // boost fades out.
            let brightness_fade = 1.0 - fade_factor;

            let multiplier =
                1.0 + (runtime_state.app.warmup_brightness_multiplier - 1.0) * brightness_fade;
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

        // Build preset comparison overlay (Shift+1-7 keys)
        let preset_comparison_lines: Option<RenderedOverlay> = if runtime_state
            .overlay_state
            .is_open(OverlayType::PresetComparison)
            && !runtime_state.overlay_state.is_open(OverlayType::Dashboard)
        {
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
        let palette_editor_overlay: Option<RenderedOverlay> = (runtime_state
            .overlay_state
            .is_palette_editor_open()
            && !runtime_state.overlay_state.is_open(OverlayType::Dashboard))
        .then(|| {
            runtime_state
                .overlay_state
                .palette_editor
                .as_ref()
                .map(|s| PaletteEditorOverlay::build_overlay(s, &runtime_state.panel_style, accent))
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
        let keyboard_hints_lines: Option<RenderedOverlay> = if runtime_state
            .overlay_state
            .is_open(OverlayType::KeyboardHints)
            && !runtime_state.overlay_state.is_open(OverlayType::Dashboard)
        {
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
            if runtime_state.overlay_state.is_open(OverlayType::Controls)
                && !runtime_state.overlay_state.is_open(OverlayType::Dashboard)
            {
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
                    runtime_state.current_color_aa().as_label(),
                    runtime_state.palette_shift_speed,
                    runtime_state.invert_palette,
                    runtime_state.reverse_palette,
                    runtime_state.dither_mode.name(),
                    term_width as usize,
                    runtime_state.default_values,
                    agent_count,
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
            Some(agent_count),
            Some(diffusion_kernel_name),
            !runtime_state.undo_stack.is_empty(),
            !runtime_state.redo_stack.is_empty(),
            Some(ui_accent),
        );
        let status_x = OverlayRenderer::status_line_x(&status_line, term_width as usize);
        // In windowed mode the expanded chrome footer replaces the status bar.
        // Only show the legacy status bar in fullscreen mode, or when explicitly
        // enabled via `show_status_bar`.
        let show_status = {
            use crate::simulation::config::ChromeStyle;
            matches!(runtime_state.chrome_style, ChromeStyle::Fullscreen)
                || runtime_state.show_status_bar
        };
        let status_data =
            if show_status && (runtime_state.any_overlay_open() || runtime_state.is_paused) {
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
        let entropy = DashboardOverlay::calculate_entropy(&blended_trail_buffer, 100);
        let trail_sum: f32 = blended_trail_buffer.iter().sum();
        let trail_capacity = sim_dims as f32 * 10.0;
        let trail_density = if trail_capacity > 0.0 {
            (trail_sum / trail_capacity).min(1.0)
        } else {
            0.0
        };

        runtime_state.update_history(timer.current_fps() as f32, entropy, trail_density);

        let dashboard_overlay: Option<RenderedOverlay> =
            if runtime_state.overlay_state.is_open(OverlayType::Dashboard) {
                let elapsed = start_time.elapsed().as_secs_f32();
                let trail_max = blended_trail_buffer.iter().fold(0.0f32, |m, &v| v.max(m));
                let memory_mb = memory_stats()
                    .map(|m| m.physical_mem as f32 / 1024.0 / 1024.0)
                    .unwrap_or(0.0);
                let frame_time_ms = timer.last_frame_ms();
                let cpu_percent = (frame_time_ms / TARGET_FRAME_TIME_MS) * 100.0;

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
                    runtime_state.app.warmup_frames,
                    runtime_state.app.auto_reset,
                    ui_accent,
                    &runtime_state.panel_style,
                ))
            } else {
                None
            };

        let (dashboard_x, dashboard_y) =
            DashboardOverlay::calculate_position(term_width as usize, term_height as usize);

        // Config browser overlay
        let config_browser_overlay: Option<RenderedOverlay> = if runtime_state
            .overlay_state
            .is_open(OverlayType::ConfigBrowser)
            && !runtime_state.overlay_state.is_open(OverlayType::Dashboard)
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
                    runtime_state.overlay_state.close();
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
            if runtime_state.overlay_state.is_open(OverlayType::ConfigSave)
                && !runtime_state.overlay_state.is_open(OverlayType::Dashboard)
            {
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

        update_food_persistence(sim, &mut runtime_state);
        check_auto_reset(sim, &mut runtime_state, entropy);

        // Update pause frame counter for animated pause effects
        if runtime_state.is_paused {
            runtime_state.pause_frame_counter += 1;
        } else {
            runtime_state.pause_frame_counter = 0;
        }

        // VCR-style pause overlay: dimmed logo centered in the drawable area
        // (the status bar shows PAUSED; no separate badge).
        let (pause_logo_overlay, pause_logo_x, pause_logo_y) = if runtime_state.is_paused
            && !runtime_state.any_overlay_open()
            && runtime_state.pause_logo_enabled
        {
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

            let actual_logo_h = logo.lines.len();
            let lx = (term_width as usize).saturating_sub(logo_w) / 2;
            // Center vertically in the drawable area (exclude status bar row)
            let drawable_h = (term_height as usize).saturating_sub(1);
            let ly = drawable_h.saturating_sub(actual_logo_h) / 2;

            (Some(logo), lx, ly)
        } else {
            (None, 0, 0)
        };

        // Update chrome snapshot so the renderer can draw expanded chrome overlays.
        {
            let diffusion_kernel_str = match runtime_state.diffusion_kernel {
                crate::simulation::config::DiffusionKernel::Mean3x3 => "Mean3x3",
                crate::simulation::config::DiffusionKernel::Gaussian => "Gaussian",
            };
            renderer.set_chrome_snapshot(ChromeSnapshot {
                chrome_state: runtime_state.chrome_state,
                preset: runtime_state.current_preset,
                palette: current_palette.clone(),
                charset_str: charset_name(&runtime_state.current_charset()).to_string(),
                population: agent_count,
                time_scale: runtime_state.time_scale,
                dither_mode: runtime_state.dither_mode,
                diffusion_kernel: Some(diffusion_kernel_str.to_string()),
                can_undo: !runtime_state.undo_stack.is_empty(),
                can_redo: !runtime_state.redo_stack.is_empty(),
                is_paused: runtime_state.is_paused,
            });
        }

        if args.species_colors_enabled() && sim.config().separate_species_trails {
            let species_trail_maps = sim.trail_maps_for_species_colors();
            let combined: Vec<_> = species_trail_maps
                .iter()
                .zip(current_species_rgb.iter())
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
                runtime_state.overlay_state.active(),
                runtime_state.pause_style,
                runtime_state.pause_pulse_draw_mode,
            )?;
        } else {
            renderer.render_with_overlay(
                downsampled_frame.cells(),
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
                runtime_state.overlay_state.active(),
                runtime_state.pause_style,
                runtime_state.pause_pulse_draw_mode,
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

                    // Screensaver mode: any key press exits (mouse events and
                    // resizes do not). Checked before overlay/control dispatch so
                    // no other handler can swallow the event.
                    if screensaver_exit_on_key(&mode, &key_event) {
                        should_exit = true;
                        break;
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
                        runtime_state.warmup_counter = runtime_state.app.warmup_frames;
                        // Skip to end
                    }

                    // Centralized overlay input handling: toggle keys, Escape, and
                    // blocking other keys while an overlay is open.
                    match OverlayInputManager::handle_input(
                        &runtime_state.overlay_state,
                        &key_event,
                    ) {
                        OverlayInputResult::CloseOverlay => {
                            runtime_state.overlay_state.close();
                            continue;
                        }
                        OverlayInputResult::Consumed => {
                            // Key was blocked by open overlay, do nothing
                            continue;
                        }
                        OverlayInputResult::NotHandled => {
                            // No overlay open, continue to normal processing
                        }
                    }

                    // Handle config save dialog input
                    if runtime_state.overlay_state.is_open(OverlayType::ConfigSave) {
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
                                    let named_profile = config_manager::NamedProfile {
                                        name: runtime_state.config_save_name_input.clone(),
                                        description: None,
                                        overrides: config_manager::capture_overrides(
                                            sim.config(),
                                            // Save the EXACT live palette/charset (incl. Custom),
                                            // not the lossy index or the stale launch charset.
                                            runtime_state.live_palette.clone(),
                                            runtime_state.live_charset.clone(),
                                            &runtime_state,
                                            // Live apply-only flags (a load may have changed them),
                                            // not the launch CLI args.
                                            runtime_state.reverse_palette,
                                            runtime_state.invert_palette,
                                            runtime_state.food_persist_enabled,
                                        ),
                                    };

                                    match config_manager::save_config(named_profile) {
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
                                runtime_state.overlay_state.close();
                                continue;
                            }
                            _ => continue,
                        }
                    }

                    // Handle preset comparison input
                    if runtime_state
                        .overlay_state
                        .is_open(OverlayType::PresetComparison)
                    {
                        use crossterm::event::KeyCode;
                        if key_event.code == KeyCode::Enter {
                            let preset = runtime_state.comparison_preset;
                            switch_preset(
                                preset,
                                &mut runtime_state,
                                &mut renderer,
                                sim,
                                &mut timer,
                                &mut grid_renderer,
                                &mut window,
                                &mut downsampled_frame,
                                &mut aux_frame,
                                (term_width as usize, term_height as usize),
                            )?;
                            runtime_state.show_notification(format!(
                                "Applied preset: {}",
                                crate::terminal::control::preset_name(preset)
                            ));
                            runtime_state.overlay_state.close();
                            continue;
                        }
                        // Note: Other keys are blocked by centralized handler
                    }

                    // Handle config browser input
                    if runtime_state
                        .overlay_state
                        .is_open(OverlayType::ConfigBrowser)
                    {
                        use crossterm::event::KeyCode;
                        match key_event.code {
                            KeyCode::Up => {
                                if runtime_state.config_browser_selected_index > 0 {
                                    runtime_state.config_browser_selected_index -= 1;
                                }
                                continue;
                            }
                            KeyCode::Down => {
                                // Bound the selection to the last config so it can't
                                // overshoot the list (render-time clamp is kept as a
                                // defensive guard).
                                let last = config_manager::list_configs()
                                    .map(|c| c.len().saturating_sub(1))
                                    .unwrap_or(0);
                                if runtime_state.config_browser_selected_index < last {
                                    runtime_state.config_browser_selected_index += 1;
                                }
                                continue;
                            }
                            KeyCode::Enter => {
                                if let Ok(configs) = config_manager::list_configs() {
                                    if let Some(config) =
                                        configs.get(runtime_state.config_browser_selected_index)
                                    {
                                        // The ONE total apply seam: applies ALL levers
                                        // (sim+render+app+flags), totally syncs the renderer,
                                        // preserves precise wind, and restarts with the
                                        // correct init + fresh/pinned seed.
                                        match crate::app::apply_overrides(
                                            &config.overrides,
                                            &mut runtime_state,
                                            &mut renderer,
                                            sim,
                                            &mut timer,
                                            &mut grid_renderer,
                                            &mut window,
                                            &mut downsampled_frame,
                                            &mut aux_frame,
                                            (term_width as usize, term_height as usize),
                                            true,
                                        ) {
                                            Ok(()) => {
                                                // Commit provenance ONLY on success so a failed
                                                // load never leaves stale active metadata.
                                                runtime_state.active_source =
                                                    crate::profile::ProfileSource::SavedConfig(
                                                        config.name.clone(),
                                                    );
                                                runtime_state.active_overrides =
                                                    config.overrides.clone();
                                                runtime_state.show_notification(format!(
                                                    "Config '{}' loaded",
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
                                runtime_state.overlay_state.close();
                                continue;
                            }
                            KeyCode::Delete => {
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
                            _ => continue,
                        }
                    }

                    // Handle palette editor input using OverlayInputHandler trait
                    if runtime_state.overlay_state.is_palette_editor_open() {
                        use crate::overlay::input::OverlayInputHandler;

                        // Initialize palette editor state if needed
                        if runtime_state.overlay_state.palette_editor.is_none() {
                            let current_palette = runtime_state.current_palette(&ALL_PALETTES);
                            runtime_state
                                .overlay_state
                                .open_palette_editor(PaletteEditorState::new(&current_palette));
                        }

                        let mut should_close = false;
                        let mut notification: Option<String> = None;

                        if let Some(ref mut state) = runtime_state.overlay_state.palette_editor {
                            let was_modified = state.is_modified;

                            let handled = state.handle_key(&key_event);

                            // Check if we need to close (Esc or Enter in Editing mode returned false)
                            let is_escape = key_event.code == crossterm::event::KeyCode::Esc;
                            let is_enter = key_event.code == crossterm::event::KeyCode::Enter;
                            let is_editing = matches!(state.mode, EditorMode::Editing);

                            if !handled && (is_escape || is_enter) && is_editing {
                                should_close = true;
                                if is_enter {
                                    notification = Some("Custom palette applied".to_string());
                                } else {
                                    // On escape, restore original palette
                                    let original = Palette::Custom(state.original_colors.to_vec());
                                    renderer.set_palette(original);
                                }
                            } else if handled {
                                // Apply palette changes after each adjustment
                                renderer.set_palette(state.to_palette());

                                if matches!(state.mode, EditorMode::SaveDialog) {
                                    // No-op: save completion is detected via the
                                    // is_modified transition below.
                                }
                            }

                            // Track if palette was saved
                            let saved_name = if was_modified && !state.is_modified {
                                Some(state.base_palette_name.clone())
                            } else {
                                None
                            };

                            // Apply cleanup after borrow
                            if should_close {
                                runtime_state.overlay_state.close_palette_editor();
                            }
                            if let Some(name) = saved_name {
                                runtime_state.saved_palette_name = Some(name);
                            }
                            if let Some(msg) = notification {
                                runtime_state.show_notification(msg);
                            }
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
                            if runtime_state.is_paused {
                                runtime_state.on_pause();
                            } else {
                                runtime_state.on_unpause_with_fade();
                            }
                        }
                        ControlAction::Restart => {
                            sim.reset(
                                runtime_state.original_seed,
                                runtime_state.original_init_mode,
                            );
                        }
                        ControlAction::SetPreset(preset) => {
                            switch_preset(
                                preset,
                                &mut runtime_state,
                                &mut renderer,
                                sim,
                                &mut timer,
                                &mut grid_renderer,
                                &mut window,
                                &mut downsampled_frame,
                                &mut aux_frame,
                                (term_width as usize, term_height as usize),
                            )?;
                        }
                        ControlAction::ComparePreset(preset) => {
                            runtime_state.toggle_preset_comparison(preset);
                            if runtime_state.any_overlay_open() {
                                runtime_state.on_modal_open();
                            } else {
                                runtime_state.on_modal_close();
                            }
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
                            renderer.set_color_aa(runtime_state.current_color_aa());
                            runtime_state.show_notification(format!(
                                "Charset: {}",
                                charset_name(&runtime_state.current_charset())
                            ));
                        }
                        ControlAction::CycleCharsetReverse => {
                            runtime_state.cycle_charset_reverse();
                            renderer.set_charset(runtime_state.current_charset());
                            renderer.set_color_aa(runtime_state.current_color_aa());
                            runtime_state.show_notification(format!(
                                "Charset: {}",
                                charset_name(&runtime_state.current_charset())
                            ));
                        }
                        ControlAction::CycleColorAa => {
                            if runtime_state.cycle_color_aa() {
                                renderer.set_color_aa(runtime_state.current_color_aa());
                                runtime_state.show_notification(format!(
                                    "Color AA ({}): {}",
                                    charset_name(&runtime_state.current_charset()),
                                    runtime_state.current_color_aa().as_label()
                                ));
                            } else {
                                runtime_state.show_notification(format!(
                                    "Color AA not applicable to {}",
                                    charset_name(&runtime_state.current_charset())
                                ));
                            }
                        }
                        ControlAction::ToggleDither => {
                            if dither_unlocked {
                                runtime_state.toggle_dither();
                                renderer.set_dither_mode(runtime_state.dither_mode);
                            } else {
                                runtime_state.show_notification(
                                    "Dither is dev-only - see help-wanted issues on GitHub"
                                        .to_string(),
                                );
                            }
                        }
                        ControlAction::CycleDitherMode => {
                            if dither_unlocked {
                                runtime_state.cycle_dither_mode();
                                renderer.set_dither_mode(runtime_state.dither_mode);
                            } else {
                                runtime_state.show_notification(
                                    "Dither is dev-only - see help-wanted issues on GitHub"
                                        .to_string(),
                                );
                            }
                        }
                        ControlAction::AdjustDitherIntensity(delta) => {
                            if dither_unlocked {
                                runtime_state.adjust_dither_intensity(delta);
                                renderer.set_dither_mode(runtime_state.dither_mode);
                            } else {
                                runtime_state.show_notification(
                                    "Dither is dev-only - see help-wanted issues on GitHub"
                                        .to_string(),
                                );
                            }
                        }
                        ControlAction::ToggleKeyboardHints => {
                            runtime_state.toggle_keyboard_hints();
                            if runtime_state.any_overlay_open() {
                                runtime_state.on_modal_open();
                            } else {
                                runtime_state.on_modal_close();
                            }
                        }
                        ControlAction::ToggleControls => {
                            runtime_state.toggle_controls();
                            if runtime_state.any_overlay_open() {
                                runtime_state.on_modal_open();
                            } else {
                                runtime_state.on_modal_close();
                            }
                        }
                        ControlAction::CloseOverlays => {
                            if runtime_state.any_overlay_open() {
                                runtime_state.close_all_overlays();
                                runtime_state.on_modal_close();
                            }
                            // If no overlays open, Esc does nothing (doesn't quit)
                        }
                        ControlAction::CycleOptionsCategory => {
                            if !runtime_state.overlay_state.is_open(OverlayType::Controls) {
                                runtime_state.toggle_controls();
                            } else {
                                runtime_state.cycle_controls_category(true);
                            }
                        }
                        ControlAction::CycleOptionsCategoryReverse => {
                            if !runtime_state.overlay_state.is_open(OverlayType::Controls) {
                                runtime_state.toggle_controls();
                            } else {
                                runtime_state.cycle_controls_category(false);
                            }
                        }
                        ControlAction::AdjustSensorAngle(delta) => {
                            let at_bound = runtime_state.adjust_sensor_angle(delta);
                            sim.with_config_mut(|c| c.sensor_angle = runtime_state.sensor_angle);
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Sensor angle at {}°",
                                    runtime_state.sensor_angle
                                ));
                            }
                        }
                        ControlAction::AdjustSensorDistance(delta) => {
                            let at_bound = runtime_state.adjust_sensor_distance(delta);
                            sim.with_config_mut(|c| {
                                c.sensor_distance = runtime_state.sensor_distance
                            });
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Sensor distance at {:.1}",
                                    runtime_state.sensor_distance
                                ));
                            }
                        }
                        ControlAction::AdjustTurnAngle(delta) => {
                            let at_bound = runtime_state.adjust_rotation_angle(delta);
                            sim.with_config_mut(|c| {
                                c.rotation_angle = runtime_state.rotation_angle
                            });
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Turn angle at {}°",
                                    runtime_state.rotation_angle
                                ));
                            }
                        }
                        ControlAction::AdjustStepSize(delta) => {
                            let at_bound = runtime_state.adjust_step_size(delta);
                            sim.with_config_mut(|c| c.step_size = runtime_state.step_size);
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Step size at {:.1}",
                                    runtime_state.step_size
                                ));
                            }
                        }
                        ControlAction::AdjustDecay(delta) => {
                            let at_bound = runtime_state.adjust_decay(delta);
                            sim.with_config_mut(|c| c.decay_factor = runtime_state.decay_factor);
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Decay factor at {:.3}",
                                    runtime_state.decay_factor
                                ));
                            }
                        }
                        ControlAction::AdjustDeposit(delta) => {
                            let at_bound = runtime_state.adjust_deposit(delta);
                            sim.with_config_mut(|c| {
                                c.deposit_amount = runtime_state.deposit_amount
                            });
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Deposit amount at {:.1}",
                                    runtime_state.deposit_amount
                                ));
                            }
                        }
                        ControlAction::CycleDiffusionKernel => {
                            runtime_state.cycle_diffusion_kernel();
                            sim.with_config_mut(|c| {
                                c.diffusion_kernel = runtime_state.diffusion_kernel
                            });
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
                            sim.with_config_mut(|c| {
                                c.diffusion_sigma = runtime_state.diffusion_sigma
                            });
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Diffusion sigma at {:.2}",
                                    runtime_state.diffusion_sigma
                                ));
                            }
                        }
                        ControlAction::AdjustAttractorStrength(delta) => {
                            let at_bound = runtime_state.adjust_attractor_strength(delta);
                            sim.with_config_mut(|c| {
                                c.attractor_strength = runtime_state.attractor_strength
                            });
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
                            // Coarse cycle PRODUCES the precise vector; store it losslessly.
                            runtime_state.wind = runtime_state.wind_direction.to_wind();
                            sim.with_config_mut(|c| c.wind = runtime_state.wind);
                            runtime_state.show_notification(format!(
                                "Wind: {}",
                                runtime_state.wind_direction.name()
                            ));
                        }
                        ControlAction::CycleWindDirectionReverse => {
                            runtime_state.cycle_wind_direction_reverse();
                            runtime_state.wind = runtime_state.wind_direction.to_wind();
                            sim.with_config_mut(|c| c.wind = runtime_state.wind);
                            runtime_state.show_notification(format!(
                                "Wind: {}",
                                runtime_state.wind_direction.name()
                            ));
                        }
                        ControlAction::AdjustTerrainStrength(delta) => {
                            let at_bound = runtime_state.adjust_terrain_strength(delta);
                            sim.with_config_mut(|c| {
                                c.terrain_strength = runtime_state.terrain_strength
                            });
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Terrain strength at {:.1}",
                                    runtime_state.terrain_strength
                                ));
                            }
                        }
                        ControlAction::CycleTerrainType => {
                            runtime_state.cycle_terrain_type();
                            sim.with_config_mut(|c| c.terrain = runtime_state.terrain_type);
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
                            if current_auto_normalize {
                                // Auto-normalize drives the white-point from the
                                // adaptive peak, so the manual control is inert.
                                runtime_state.show_notification(
                                    "Brightness is auto-normalized (press B to disable)"
                                        .to_string(),
                                );
                            } else {
                                let at_bound = runtime_state.adjust_max_brightness(delta);
                                // Mirror the live value into the sim config so it is
                                // captured on save (Ctrl+S reads sim.config()), matching
                                // how every other adjustable parameter syncs.
                                sim.with_config_mut(|c| {
                                    c.max_brightness = runtime_state.max_brightness;
                                });
                                if at_bound {
                                    let gain = crate::config_defaults::trail::brightness_gain(
                                        runtime_state.max_brightness,
                                    );
                                    runtime_state
                                        .show_notification(format!("Brightness at {gain:.1}×"));
                                }
                            }
                        }
                        ControlAction::SaveFrameToPng => {
                            use crate::export::png::save_frame_as_png;

                            let png_aux_cells = if runtime_state.temporal_color > 0.0 {
                                Some(aux_frame.cells.as_slice())
                            } else {
                                None
                            };
                            match save_frame_as_png(
                                downsampled_frame.cells(),
                                term_width as usize,
                                term_height as usize,
                                current_palette.clone(),
                                runtime_state.reverse_palette,
                                runtime_state.invert_palette,
                                hue_offset,
                                Some(&runtime_state.intensity_mapping),
                                max_brightness.max(1.0),
                                runtime_state.temporal_color,
                                runtime_state.temporal_mode,
                                png_aux_cells,
                                runtime_state.palette_cycle,
                                None,
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
                            let ov = runtime_state.active_overrides.clone();
                            runtime_state.reset_transient();
                            match crate::app::apply_overrides(
                                &ov,
                                &mut runtime_state,
                                &mut renderer,
                                sim,
                                &mut timer,
                                &mut grid_renderer,
                                &mut window,
                                &mut downsampled_frame,
                                &mut aux_frame,
                                (term_width as usize, term_height as usize),
                                true,
                            ) {
                                Ok(()) => {
                                    hue_offset = 0.0;
                                    current_auto_normalize = runtime_state.auto_normalize;
                                    runtime_state
                                        .show_notification("Reset to defaults".to_string());
                                }
                                Err(e) => {
                                    runtime_state.show_notification(format!("Reset failed: {e}"));
                                }
                            }
                        }
                        ControlAction::ToggleDashboard => {
                            runtime_state.toggle_dashboard();
                            if runtime_state.any_overlay_open() {
                                runtime_state.on_modal_open();
                            } else {
                                runtime_state.on_modal_close();
                            }
                        }
                        ControlAction::SetIntensityMapping(_) => {}
                        ControlAction::ShowConfigBrowser => {
                            runtime_state.close_all_overlays();
                            runtime_state.overlay_state.open(OverlayType::ConfigBrowser);
                            runtime_state.config_browser_selected_index = 0;
                            runtime_state.on_modal_open();
                        }
                        ControlAction::ShowConfigSaveDialog => {
                            runtime_state.close_all_overlays();
                            runtime_state.overlay_state.open(OverlayType::ConfigSave);
                            runtime_state.config_save_name_input.clear();
                            runtime_state.on_modal_open();
                        }
                        ControlAction::RandomizeParams => {
                            runtime_state.randomize_params();
                            apply_random_config(&runtime_state, sim, &mut renderer, &ALL_PALETTES);

                            runtime_state.show_notification("Parameters Randomized!".to_string());
                        }
                        ControlAction::Undo => {
                            if runtime_state.undo().is_some() {
                                apply_live_params(&runtime_state, sim, &mut renderer);
                                runtime_state.show_notification("Undo successful".to_string());
                            } else {
                                runtime_state.show_notification("Nothing to undo".to_string());
                            }
                        }
                        ControlAction::Redo => {
                            if runtime_state.redo().is_some() {
                                apply_live_params(&runtime_state, sim, &mut renderer);
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
                        ControlAction::CycleWindowFrame => {
                            runtime_state.cycle_window_frame();
                            renderer.set_window_frame(runtime_state.window_frame);
                            runtime_state.show_notification(format!(
                                "Frame: {:?}",
                                runtime_state.window_frame
                            ));
                        }
                        ControlAction::CycleWindowFrameReverse => {
                            runtime_state.cycle_window_frame_reverse();
                            renderer.set_window_frame(runtime_state.window_frame);
                            runtime_state.show_notification(format!(
                                "Frame: {:?}",
                                runtime_state.window_frame
                            ));
                        }
                        ControlAction::ShowPaletteEditor => {
                            if runtime_state.overlay_state.is_palette_editor_open() {
                                runtime_state.overlay_state.close_palette_editor();
                            } else {
                                let current_palette = runtime_state.current_palette(&ALL_PALETTES);
                                runtime_state
                                    .overlay_state
                                    .open_palette_editor(PaletteEditorState::new(&current_palette));
                            }
                            if runtime_state.any_overlay_open() {
                                runtime_state.on_modal_open();
                            } else {
                                runtime_state.on_modal_close();
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
                        ControlAction::CycleChrome => {
                            use crate::render::window::FallbackMode;
                            use crate::simulation::config::ChromeStyle;
                            runtime_state.cycle_chrome_style();
                            let layout =
                                if matches!(runtime_state.chrome_style, ChromeStyle::Fullscreen) {
                                    None
                                } else {
                                    let (tw, th) = crossterm::terminal::size()
                                        .map(|(w, h)| (w as usize, h as usize))
                                        .unwrap_or((80, 24));
                                    let l = window.compute_rects(tw, th);
                                    if matches!(l.fallback, FallbackMode::Fullscreen) {
                                        None
                                    } else {
                                        Some(l)
                                    }
                                };
                            renderer.set_window_layout(layout);
                            runtime_state.show_notification(format!(
                                "Chrome: {:?}",
                                runtime_state.chrome_style
                            ));
                        }
                        ControlAction::ToggleFullscreen => {
                            use crate::render::window::FallbackMode;
                            use crate::simulation::config::ChromeStyle;
                            if matches!(runtime_state.chrome_style, ChromeStyle::Fullscreen) {
                                // Restore windowed mode
                                runtime_state.chrome_style = ChromeStyle::Minimal;
                                let (tw, th) = crossterm::terminal::size()
                                    .map(|(w, h)| (w as usize, h as usize))
                                    .unwrap_or((80, 24));
                                let l = window.compute_rects(tw, th);
                                let layout = if matches!(l.fallback, FallbackMode::Fullscreen) {
                                    None
                                } else {
                                    Some(l)
                                };
                                renderer.set_window_layout(layout);
                            } else {
                                // Switch to fullscreen
                                runtime_state.chrome_style = ChromeStyle::Fullscreen;
                                runtime_state.chrome_state =
                                    crate::terminal::state::ChromeState::Minimal;
                                renderer.set_window_layout(None);
                            }
                        }
                        #[cfg(feature = "audio")]
                        ControlAction::ToggleChoir => {
                            if choir.is_some() {
                                choir = None;
                                runtime_state.show_notification("Choir mode: off".to_string());
                            } else {
                                match crate::audio::Choir::try_new(
                                    args.choir_volume.clamp(0.0, 1.0),
                                ) {
                                    Ok(c) => {
                                        choir = Some(c);
                                        runtime_state
                                            .show_notification("Choir mode: on".to_string());
                                    }
                                    Err(e) => {
                                        runtime_state
                                            .show_notification(format!("Choir init failed: {e}"));
                                    }
                                }
                            }
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

            // Update pause frame counter for animated pause effects
            if runtime_state.is_paused {
                runtime_state.pause_frame_counter += 1;
            } else {
                runtime_state.pause_frame_counter = 0;
            }

            // Rebuild pause overlays with new state
            let (pause_logo_overlay, pause_logo_x, pause_logo_y) =
                if runtime_state.is_paused && runtime_state.pause_logo_enabled {
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

                    let actual_logo_h = logo.lines.len();
                    let lx = (term_width as usize).saturating_sub(logo_w) / 2;
                    let drawable_h = (term_height as usize).saturating_sub(1);
                    let ly = drawable_h.saturating_sub(actual_logo_h) / 2;

                    (Some(logo), lx, ly)
                } else {
                    (None, 0, 0)
                };

            // Build status line
            let diffusion_kernel_name = match current_diffusion_kernel {
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
            // Suppress status bar in windowed mode unless explicitly enabled.
            let show_status_pause = {
                use crate::simulation::config::ChromeStyle;
                matches!(runtime_state.chrome_style, ChromeStyle::Fullscreen)
                    || runtime_state.show_status_bar
            };
            let status_data = if show_status_pause {
                Some((status_line, status_x, status_colors))
            } else {
                None
            };

            // Re-render with updated pause state
            if args.species_colors_enabled() && sim.config().separate_species_trails {
                let species_trail_maps = sim.trail_maps_for_species_colors();
                let combined: Vec<_> = species_trail_maps
                    .iter()
                    .zip(current_species_rgb.iter())
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
                    runtime_state.overlay_state.active(),
                    runtime_state.pause_style,
                    runtime_state.pause_pulse_draw_mode,
                )?;
            } else {
                renderer.render_with_overlay(
                    downsampled_frame.cells(),
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
                    runtime_state.overlay_state.active(),
                    runtime_state.pause_style,
                    runtime_state.pause_pulse_draw_mode,
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

/// Live preset-switch: applies the bare preset through the ONE total apply seam,
/// wiping the trail (restart: true). CLI args are NOT sticky after a swap.
#[allow(clippy::too_many_arguments)]
fn switch_preset(
    new_preset: Preset,
    rs: &mut RuntimeState,
    renderer: &mut TerminalRenderer,
    sim: &mut Simulation,
    timer: &mut FrameTimer,
    grid_renderer: &mut Option<crate::render::grid::GridRenderer>,
    window: &mut crate::render::window::Window,
    downsampled_frame: &mut crate::render::downsample::DownsampledFrame,
    aux_frame: &mut crate::render::downsample::AuxFrame,
    term_size: (usize, usize),
) -> io::Result<()> {
    let ov = crate::profile_overrides::ProfileOverrides {
        preset: Some(new_preset),
        ..Default::default()
    };
    crate::app::apply_overrides(
        &ov,
        rs,
        renderer,
        sim,
        timer,
        grid_renderer,
        window,
        downsampled_frame,
        aux_frame,
        term_size,
        true,
    )
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    // Commit provenance ONLY after successful apply (transactional).
    rs.set_preset(new_preset);
    rs.active_source = crate::profile::ProfileSource::Preset(new_preset);
    rs.active_overrides = ov;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    fn key(code: KeyCode, kind: KeyEventKind) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyModifiers::NONE,
            kind,
            state: KeyEventState::NONE,
        }
    }

    #[test]
    fn screensaver_exits_on_any_key_press() {
        for code in [
            KeyCode::Char('a'),
            KeyCode::Char(' '),
            KeyCode::Enter,
            KeyCode::Esc,
            KeyCode::Up,
            KeyCode::F(1),
        ] {
            assert!(
                screensaver_exit_on_key(&Mode::Screensaver, &key(code, KeyEventKind::Press)),
                "expected screensaver exit for {:?}",
                code
            );
        }
    }

    #[test]
    fn screensaver_ignores_key_release_and_repeat() {
        let release = key(KeyCode::Char('a'), KeyEventKind::Release);
        let repeat = key(KeyCode::Char('a'), KeyEventKind::Repeat);
        assert!(!screensaver_exit_on_key(&Mode::Screensaver, &release));
        assert!(!screensaver_exit_on_key(&Mode::Screensaver, &repeat));
    }

    #[test]
    fn non_screensaver_modes_never_exit_on_key() {
        let press = key(KeyCode::Char('a'), KeyEventKind::Press);
        for mode in [Mode::Default, Mode::Live, Mode::Print] {
            assert!(!screensaver_exit_on_key(&mode, &press));
        }
    }
}
