// Imports are shared unevenly between mod.rs and runner.rs; splitting them
// cleanly between the two files is deferred.
#![allow(unused_imports)]

use std::io::{self, Write};

use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};

use crate::cli::{self, Args, ColorMode, Mode};
use crate::exploration::{Explorer, ExplorerConfig, PresetBehavior};
use crate::export::GifExporter;
use crate::export::WebmExporter;
use crate::render::adaptive_brightness::AdaptiveBrightness;
use crate::render::charset::Charset;
use crate::render::dither::DitherMode;
use crate::render::downsample::{downsample, DownsampledFrame};
use crate::render::grid::{GridRenderer, GridStyle};
use crate::render::overlay::{
    build_notification_panel, ConfigBrowserOverlay, ConfigSaveOverlay, DashboardOverlay,
    KeyboardHintsOverlay, PauseOverlay, PresetComparisonOverlay, RenderedOverlay,
};
use crate::render::palette::{hex_to_rgb, palette_accent_color, RgbColor};
use crate::render::palette_editor::{
    EditorComponent, EditorMode, PaletteEditorOverlay, PaletteEditorState,
};
use crate::simulation;
use crate::simulation::config::{
    Attractor, DiffusionKernel, InitMode, Preset, SimConfig, TerrainType,
};
use crate::simulation::Simulation;
use crate::terminal::control::{
    charset_name, handle_key_event, num_palettes, palette_name, preset_name, ControlAction,
    MouseInteractionMode, PaletteShiftSpeed, RuntimeState, ALL_CHARSETS, ALL_PALETTES,
};
use crate::terminal::detection::{log_capabilities, TerminalCapabilities};
use crate::terminal::frame_buffer::FrameBuffer;
use crate::terminal::input::{InputPoller, MouseEventType};
use crate::terminal::renderer::TerminalRenderer;
use crate::terminal::screen::TerminalScreen;
use crate::terminal::signal::is_shutdown_requested;
use crate::terminal::timing::FrameTimer;

const REFERENCE_TIME_STEP: f32 = 1.0 / 30.0;

/// Parameter explanations and help text.
pub mod explanations;
/// Interactive simulation runner.
pub mod runner;

pub use explanations::print_parameter_explanations;
pub use runner::{get_terminal_size, run_simulation};

/// Fold afterglow into a blended trail buffer in place, before downsampling.
/// `glow_mix == 0.0` (or `lag == None`) is a no-op (byte-identical). Adds
/// `glow_mix · lag[i]` to each cell so the glow participates in the white-point.
pub(crate) fn fold_afterglow(blended: &mut [f32], lag: Option<&[f32]>, glow_mix: f32) {
    if glow_mix <= 0.0 {
        return;
    }
    if let Some(lag) = lag {
        for (b, &l) in blended.iter_mut().zip(lag.iter()) {
            *b += glow_mix * l;
        }
    }
}

/// Extracts each species' RGB color from the config, in species order.
pub fn extract_species_rgb_colors(config: &SimConfig) -> Vec<RgbColor> {
    config.species_configs.iter().map(|s| s.color).collect()
}

/// Pushes every renderer-layer cache from runtime state into the renderer.
///
/// This is the single source of truth for "what the renderer should reflect after a
/// runtime-state change" (load / undo / redo / randomize / reset). Keeping it in one place
/// prevents the per-handler drift that left charset and intensity-mapping unapplied on load.
pub fn sync_renderer_caches(runtime_state: &RuntimeState, renderer: &mut TerminalRenderer) {
    renderer.set_palette(runtime_state.current_palette(&ALL_PALETTES));
    renderer.set_invert_palette(runtime_state.invert_palette);
    renderer.set_reverse_palette(runtime_state.reverse_palette);
    renderer.set_charset(runtime_state.current_charset());
    renderer.set_intensity_mapping(Some(runtime_state.intensity_mapping.clone()));
    renderer.set_palette_cycle(runtime_state.palette_cycle);
    renderer.set_glyph(runtime_state.glyph);
    renderer.set_window_frame(runtime_state.window_frame);
    renderer.set_dither_mode(runtime_state.dither_mode);
}

/// Overlays every live (non-restart-required) parameter from runtime state onto the
/// simulation config and renderer caches. Restart-required fields (population, grid size,
/// init mode) are deliberately untouched — see issue #46.
///
/// This is the shared path for load / undo / redo / randomize. `update_config` regenerates
/// existing trail-map kernels, so diffusion_sigma takes effect here without a separate call.
pub fn apply_live_params(
    runtime_state: &RuntimeState,
    sim: &mut Simulation,
    renderer: &mut TerminalRenderer,
) {
    let mut new_config = sim.config().clone();
    new_config.sensor_angle = runtime_state.sensor_angle;
    new_config.sensor_distance = runtime_state.sensor_distance;
    new_config.rotation_angle = runtime_state.rotation_angle;
    new_config.step_size = runtime_state.step_size;
    new_config.decay_factor = runtime_state.decay_factor;
    new_config.deposit_amount = runtime_state.deposit_amount;
    new_config.diffusion_kernel = runtime_state.diffusion_kernel;
    new_config.diffusion_sigma = runtime_state.diffusion_sigma;
    new_config.diffuse_weight = runtime_state.diffuse_weight;
    new_config.decay_gamma = runtime_state.decay_gamma;
    new_config.max_brightness = runtime_state.max_brightness;
    new_config.terrain = runtime_state.terrain_type;
    new_config.terrain_strength = runtime_state.terrain_strength;
    new_config.attractor_strength = runtime_state.attractor_strength;
    new_config.wind = runtime_state.wind_direction.to_wind();
    sim.update_config(new_config);

    sync_renderer_caches(runtime_state, renderer);
}

/// Applies randomized configuration parameters to the simulation and runtime state.
///
/// This updates the simulation configuration (sensors, movement, decay, etc.),
/// generates random attractors and obstacles, and updates the renderer's palette.
pub fn apply_random_config(
    runtime_state: &RuntimeState,
    sim: &mut Simulation,
    renderer: &mut TerminalRenderer,
    _palette_list: &[cli::Palette; cli::NUM_PALETTES],
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Apply all live params (sim config + renderer caches) through the shared path.
    apply_live_params(runtime_state, sim, renderer);

    // Then layer randomization-only extras (attractors / obstacles) on top.
    let mut new_config = sim.config().clone();

    new_config.attractors.clear();
    if rng.gen_bool(0.5) {
        let num_attractors = rng.gen_range(1..5);
        for _ in 0..num_attractors {
            new_config
                .attractors
                .push(simulation::config::Attractor::new(
                    rng.gen_range(0.0..sim.width() as f32),
                    rng.gen_range(0.0..sim.height() as f32),
                    rng.gen_range(-2.0..2.0),
                ));
        }
    }

    new_config.obstacles.clear();
    if rng.gen_bool(0.4) {
        let num_obstacles = rng.gen_range(1..4);
        for _ in 0..num_obstacles {
            if rng.gen_bool(0.5) {
                new_config
                    .obstacles
                    .push(simulation::config::Obstacle::Circle {
                        x: rng.gen_range(0.0..sim.width() as f32),
                        y: rng.gen_range(0.0..sim.height() as f32),
                        radius: rng.gen_range(10.0..40.0),
                    });
            } else {
                new_config
                    .obstacles
                    .push(simulation::config::Obstacle::Rect {
                        x: rng.gen_range(0.0..sim.width() as f32 * 0.8),
                        y: rng.gen_range(0.0..sim.height() as f32 * 0.8),
                        width: rng.gen_range(20.0..60.0),
                        height: rng.gen_range(20.0..60.0),
                    });
            }
        }
    }
    let _ = new_config.load_obstacle_masks();

    sim.update_config(new_config);
}

/// Generate shell completions and print to stdout
pub fn generate_completions(shell: &str) -> io::Result<()> {
    let shell = match shell.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        "powershell" | "pwsh" => Shell::PowerShell,
        "elvish" => Shell::Elvish,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Unknown shell: {}. Supported: bash, zsh, fish, powershell, elvish",
                    shell
                ),
            ));
        }
    };

    let mut cmd = Args::command();
    generate(shell, &mut cmd, "tslime", &mut io::stdout());
    Ok(())
}

/// Run parameter space exploration to find optimal presets.
fn run_exploration(args: &Args) -> io::Result<()> {
    let seed = args.seed.unwrap_or(42);

    let width = 200;
    let height = 200;
    let warmup_frames = 100;
    let measurement_frames = 50;

    let config = ExplorerConfig {
        width,
        height,
        warmup_frames,
        measurement_frames,
        seed,
    };

    let mut explorer = Explorer::new(config);

    // Parse target behavior
    let target_behavior = match args.explore_behavior.as_deref() {
        Some("vortex") => Some(PresetBehavior::Vortex),
        Some("lightning") => Some(PresetBehavior::Lightning),
        Some("crystal") => Some(PresetBehavior::Crystal),
        Some("blob") => Some(PresetBehavior::Blob),
        Some("worm") => Some(PresetBehavior::Worm),
        Some("chaosedge") | Some("chaos") => Some(PresetBehavior::ChaosEdge),
        Some("all") | None => None,
        Some(other) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Unknown behavior: {}. Use: vortex, lightning, crystal, blob, worm, chaosedge, all",
                    other
                ),
            ));
        }
    };

    println!("=== Parameter Space Exploration ===");
    println!("Grid size: {}x{}", width, height);
    println!("Warmup frames: {}", warmup_frames);
    println!("Measurement frames: {}", measurement_frames);
    println!("Iterations per behavior: {}", args.explore_iterations);
    println!();

    // Use hybrid search: 50% random exploration, 50% hill-climb refinement
    let random_iters = args.explore_iterations / 2;
    let hill_climb_iters = args.explore_iterations / 2;
    let top_k = 3; // Refine top 3 candidates

    if let Some(behavior) = target_behavior {
        println!("Optimizing for: {:?} (hybrid search)", behavior);
        println!("  Random exploration: {} iterations", random_iters);
        println!(
            "  Hill-climb refinement: {} iterations x {} candidates",
            hill_climb_iters, top_k
        );
        println!();

        let result = explorer.hybrid_search(behavior, random_iters, hill_climb_iters, top_k);

        println!();
        println!("=== Best Result for {:?} ===", behavior);
        println!("Score: {:.4}", behavior.score(&result.metrics));
        println!();
        println!("Metrics:");
        println!("  Angular momentum: {:.4}", result.metrics.angular_momentum);
        println!("  Heading variance: {:.4}", result.metrics.heading_variance);
        println!(
            "  Trail fragmentation: {}",
            result.metrics.trail_fragmentation
        );
        println!("  Trail elongation: {:.4}", result.metrics.trail_elongation);
        println!("  Spatial entropy: {:.4}", result.metrics.spatial_entropy);
        println!(
            "  Temporal stability: {:.4}",
            result.metrics.temporal_stability
        );
        println!("  Density variance: {:.4}", result.metrics.density_variance);
        println!("  Coverage: {:.4}", result.metrics.coverage);
        println!("  Branching factor: {:.4}", result.metrics.branching_factor);
        println!("  Flow coherence: {:.4}", result.metrics.flow_coherence);
        println!(
            "  Spatial concentration: {:.4}",
            result.metrics.spatial_concentration
        );
        println!("  Path continuity: {:.4}", result.metrics.path_continuity);
        println!();
        println!("Optimal Parameters:");
        println!("  sensor_angle: {:.1}", result.params.sensor_angle);
        println!("  sensor_distance: {:.1}", result.params.sensor_distance);
        println!("  rotation_angle: {:.1}", result.params.rotation_angle);
        println!("  step_size: {:.2}", result.params.step_size);
        println!("  decay_factor: {:.3}", result.params.decay_factor);
        println!("  deposit_amount: {:.1}", result.params.deposit_amount);
        println!("  population: {}", result.params.population);
        println!("  diffusion_kernel: {:?}", result.params.diffusion_kernel);
        println!(
            "  wind: {:?}",
            result.params.wind_dx.zip(result.params.wind_dy)
        );
        println!("  terrain: {:?}", result.params.terrain);
        println!("  terrain_strength: {:.2}", result.params.terrain_strength);
        println!("  init_mode: {:?}", result.params.init_mode);
        println!();
        println!("Rust code:");
        println!("{}", result.params.to_rust_code(&format!("{:?}", behavior)));
    } else {
        println!("Optimizing all behaviors using hybrid search:");
        println!("  Random exploration: {} iterations", random_iters);
        println!(
            "  Hill-climb refinement: {} iterations x {} candidates",
            hill_climb_iters, top_k
        );
        println!();

        let results = explorer.optimize_all_hybrid(random_iters, hill_climb_iters, top_k);

        println!();
        println!("=== Summary of All Optimized Presets ===");
        println!();

        for (behavior, result) in results {
            println!(
                "--- {:?} (score: {:.4}) ---",
                behavior,
                behavior.score(&result.metrics)
            );
            println!("  sensor_angle: {:.1}", result.params.sensor_angle);
            println!("  sensor_distance: {:.1}", result.params.sensor_distance);
            println!("  rotation_angle: {:.1}", result.params.rotation_angle);
            println!("  step_size: {:.2}", result.params.step_size);
            println!("  decay_factor: {:.3}", result.params.decay_factor);
            println!("  deposit_amount: {:.1}", result.params.deposit_amount);
            println!("  population: {}", result.params.population);
            println!("  diffusion_kernel: {:?}", result.params.diffusion_kernel);
            println!(
                "  wind: {:?}",
                result.params.wind_dx.zip(result.params.wind_dy)
            );
            println!("  terrain: {:?}", result.params.terrain);
            println!("  init_mode: {:?}", result.params.init_mode);
            println!("  flow_coherence: {:.4}", result.metrics.flow_coherence);
            println!(
                "  spatial_concentration: {:.4}",
                result.metrics.spatial_concentration
            );
            println!("  path_continuity: {:.4}", result.metrics.path_continuity);
            println!();
        }
    }

    Ok(())
}

/// Main entry point for the application logic.
///
/// Parses command-line arguments, validates them, and dispatches to the
/// appropriate mode handler (Live, Screensaver, Print, Capture, Export).
pub fn run() -> io::Result<()> {
    let args = Args::parse();

    // Informational flags print and exit before arg validation.
    if let Some(shell) = &args.completions {
        generate_completions(shell)?;
        return Ok(());
    }

    if args.explain {
        print_parameter_explanations();
        return Ok(());
    }

    if args.explore {
        run_exploration(&args)?;
        return Ok(());
    }

    args.validate()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let config = args
        .to_sim_config()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let palette = args
        .palette()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let charset = Charset::from_args(&args);

    let seed = args.seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    });

    let mut init_mode = args.init.unwrap_or(InitMode::Food);

    if args.init.is_none() {
        if let Some(preferred) = config.preferred_init_mode {
            init_mode = preferred;
        }
    }

    let mut sim = Simulation::new(
        args.resolution.width,
        args.resolution.height,
        config,
        seed,
        init_mode,
        args.effective_trail_history(),
    );

    let mode = args.mode();

    if mode == Mode::Print {
        print_mode(&mut sim, &args, palette, charset)?;
    } else if mode == Mode::CaptureFrames {
        capture_frames_mode(&mut sim, &args, palette, charset)?;
    } else if mode == Mode::GifExport {
        export_gif_mode(&mut sim, &args, palette)?;
    } else if mode == Mode::WebmExport {
        export_webm_mode(&mut sim, &args, palette)?;
    } else {
        run_simulation(&mut sim, &args, mode, palette, charset)?;
    }

    Ok(())
}

/// Executes the "Print" mode.
///
/// Advances the simulation one step and writes a single frame to stdout,
/// then exits. Useful for generating static images or piping output.
pub fn print_mode(
    sim: &mut Simulation,
    args: &Args,
    palette: cli::Palette,
    charset: Charset,
) -> io::Result<()> {
    #[cfg(windows)]
    let _enable = enable_ansi_support::enable_ansi_support();

    // Enable temporal computation if requested, then warm up enough frames
    // to populate the EMA lag buffer before capturing the final frame.
    let temporal_strength = args.temporal_color;
    let temporal_mode = match args.temporal_mode.to_ascii_lowercase().as_str() {
        "accent" => crate::render::palette::TemporalMode::Accent,
        _ => crate::render::palette::TemporalMode::Hue,
    };
    if temporal_strength > 0.0 {
        let temporal_alpha = if args.temporal_lag > 0.0 {
            1.0 / args.temporal_lag
        } else {
            1.0
        };
        sim.set_compute_temporal(true, temporal_alpha);
        // Warm up enough frames so the EMA lag buffer is populated before we
        // capture the golden frame (lag_frames warmup gives a representative diff).
        let warmup = (args.temporal_lag.ceil() as usize).max(1);
        for _ in 0..warmup {
            sim.update(1.0);
        }
    }
    if args.afterglow > 0.0 {
        sim.set_compute_afterglow(true, args.afterglow_rate);
    }

    sim.update(1.0);

    let (term_width, term_height) = get_terminal_size();

    let sim_width = sim.width();
    let sim_height = sim.height();
    let mut blended_trail = Vec::new();
    sim.trail_map_blended(&mut blended_trail);
    fold_afterglow(&mut blended_trail, sim.afterglow_lag(), args.afterglow);
    let mut downsampled = DownsampledFrame::new(term_width, term_height);
    downsample(
        &blended_trail,
        sim_width,
        sim_height,
        term_width,
        term_height,
        &mut downsampled,
    );

    // Populate auxiliary frame with temporal diff data when temporal color is active.
    let mut aux_storage = crate::render::downsample::AuxFrame {
        width: 0,
        height: 0,
        cells: Vec::new(),
    };
    let opt_aux_frame = if temporal_strength > 0.0 {
        crate::render::downsample::downsample_aux(
            None,
            None,
            None,
            sim.temporal_diff(),
            sim_width,
            sim_height,
            term_width,
            term_height,
            &mut aux_storage,
        );
        Some(&aux_storage)
    } else {
        None
    };

    let config = args
        .to_sim_config()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let color_mode = args.color_mode().unwrap_or(ColorMode::Bits256);

    let mut adaptive_brightness =
        AdaptiveBrightness::new(args.normalize_window, args.auto_normalize);
    adaptive_brightness.update(downsampled.cells());
    let max_brightness = if args.auto_normalize {
        adaptive_brightness.get_max_brightness()
    } else {
        config.max_brightness
    };

    let species_rgb_colors = if args.species_colors_enabled() {
        Some(extract_species_rgb_colors(&config))
    } else {
        None
    };

    let background_color = config.background_color.as_ref().and_then(|c| hex_to_rgb(c));

    let dither_mode = args.dither_mode().unwrap_or(DitherMode::None);
    let intensity_mapping = args
        .to_render_art_defaults()
        .ok()
        .map(|a| a.intensity_mapping);

    let palette_cycle = args
        .to_render_art_defaults()
        .ok()
        .map(|a| a.palette_cycle)
        .unwrap_or_default();

    let glyph = args
        .to_render_art_defaults()
        .ok()
        .map(|a| a.glyph)
        .unwrap_or_default();

    let mut buffer = FrameBuffer::from_downsampled(
        downsampled.cells(),
        term_width,
        term_height,
        max_brightness,
        palette.clone(),
        charset,
        args.reverse_palette,
        args.invert_palette,
        color_mode,
        0.0,
        dither_mode,
        &mut None,
        intensity_mapping.as_ref(),
        args.species_colors_enabled(),
        species_rgb_colors,
        background_color,
        args.ascii_contrast,
        opt_aux_frame,
        false,
        false,
        60.0,
        1.0,
        0.5,
        false,
        0.3,
        crate::config_defaults::TrailAgeMode::Bidirectional,
        false,
        temporal_strength,
        temporal_mode,
        palette_cycle,
        glyph,
    );

    if args.grid {
        let grid_style = args.grid_style.parse().unwrap_or(GridStyle::Cross);
        let grid_color = hex_to_rgb(&args.grid_color).unwrap_or(RgbColor {
            r: 255,
            g: 255,
            b: 255,
        });
        let mut grid_renderer = GridRenderer::new(
            grid_style,
            args.grid_size,
            grid_color,
            args.grid_opacity,
            args.grid_adaptive,
        );
        grid_renderer.initialize(term_width, term_height);

        // Calculate average brightness for adaptive opacity
        let total_brightness: f32 = downsampled
            .cells()
            .iter()
            .map(|cell| cell.top.max(cell.bottom))
            .sum();
        let avg_brightness = if !downsampled.cells().is_empty() && max_brightness > 0.0 {
            (total_brightness / (downsampled.cells().len() as f32)) / max_brightness
        } else {
            0.0
        };

        for y in 0..term_height {
            for x in 0..term_width {
                if grid_renderer.is_grid_position(x, y, term_width, term_height) {
                    let (on_vertical, on_horizontal) = grid_renderer.get_grid_lines(x, y);
                    let opacity = grid_renderer.calculate_opacity(
                        x,
                        y,
                        term_width,
                        term_height,
                        avg_brightness,
                    );
                    buffer.render_grid_background(
                        x,
                        y,
                        grid_color,
                        opacity,
                        on_vertical,
                        on_horizontal,
                    );
                }
            }
        }
    }

    print!(
        "{}",
        buffer.build_frame_string(args.plain_output, color_mode)
    );
    io::stdout().flush()?;

    Ok(())
}

/// Executes the "Capture Frames" mode.
///
/// Runs the simulation and saves a sequence of frames as text files to a
/// specified directory. Also generates a metadata JSON file.
pub fn capture_frames_mode(
    sim: &mut Simulation,
    args: &Args,
    palette: cli::Palette,
    charset: Charset,
) -> io::Result<()> {
    let (term_width, term_height) = get_terminal_size();

    std::fs::create_dir_all(&args.frame_dir)?;

    eprintln!(
        "Capturing {} frames to {}...",
        args.frame_count, args.frame_dir
    );

    let config = args
        .to_sim_config()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let color_mode = args.color_mode().unwrap_or(ColorMode::Bits256);

    let mut adaptive_brightness =
        AdaptiveBrightness::new(args.normalize_window, args.auto_normalize);

    // Temporal-color setup: enable EMA computation once before the loop.
    let temporal_strength = args.temporal_color;
    let temporal_mode = match args.temporal_mode.to_ascii_lowercase().as_str() {
        "accent" => crate::render::palette::TemporalMode::Accent,
        _ => crate::render::palette::TemporalMode::Hue,
    };
    if temporal_strength > 0.0 {
        let temporal_alpha = if args.temporal_lag > 0.0 {
            1.0 / args.temporal_lag
        } else {
            1.0
        };
        sim.set_compute_temporal(true, temporal_alpha);
    }
    if args.afterglow > 0.0 {
        sim.set_compute_afterglow(true, args.afterglow_rate);
    }

    // Reused across frames so trail_map_blended doesn't reallocate per frame.
    let mut blended_trail = Vec::new();
    // Reused aux frame for temporal diff downsampling.
    let mut aux_storage = crate::render::downsample::AuxFrame {
        width: 0,
        height: 0,
        cells: Vec::new(),
    };

    for frame_idx in 0..args.frame_count {
        for _ in 0..args.frame_skip {
            sim.update(1.0);
        }

        let sim_width = sim.width();
        let sim_height = sim.height();
        sim.trail_map_blended(&mut blended_trail);
        fold_afterglow(&mut blended_trail, sim.afterglow_lag(), args.afterglow);
        let mut downsampled = DownsampledFrame::new(term_width, term_height);
        downsample(
            &blended_trail,
            sim_width,
            sim_height,
            term_width,
            term_height,
            &mut downsampled,
        );

        adaptive_brightness.update(downsampled.cells());
        let max_brightness = if args.auto_normalize {
            adaptive_brightness.get_max_brightness()
        } else {
            config.max_brightness
        };

        let species_rgb_colors = if args.species_colors_enabled() {
            Some(extract_species_rgb_colors(&config))
        } else {
            None
        };

        let background_color = config.background_color.as_ref().and_then(|c| hex_to_rgb(c));
        let intensity_mapping = args
            .to_render_art_defaults()
            .ok()
            .map(|a| a.intensity_mapping);
        let palette_cycle_inner = args
            .to_render_art_defaults()
            .ok()
            .map(|a| a.palette_cycle)
            .unwrap_or_default();

        let glyph_inner = args
            .to_render_art_defaults()
            .ok()
            .map(|a| a.glyph)
            .unwrap_or_default();

        let opt_aux_frame = if temporal_strength > 0.0 {
            crate::render::downsample::downsample_aux(
                None,
                None,
                None,
                sim.temporal_diff(),
                sim_width,
                sim_height,
                term_width,
                term_height,
                &mut aux_storage,
            );
            Some(&aux_storage)
        } else {
            None
        };

        let mut buffer = FrameBuffer::from_downsampled(
            downsampled.cells(),
            term_width,
            term_height,
            max_brightness,
            palette.clone(),
            charset.clone(),
            args.reverse_palette,
            args.invert_palette,
            color_mode,
            0.0,
            args.dither_mode().unwrap_or(DitherMode::None),
            &mut None,
            intensity_mapping.as_ref(),
            args.species_colors_enabled(),
            species_rgb_colors,
            background_color,
            args.ascii_contrast,
            opt_aux_frame,
            false,
            false,
            60.0,
            1.0,
            0.5,
            false,
            0.3,
            crate::config_defaults::TrailAgeMode::Bidirectional,
            false,
            temporal_strength,
            temporal_mode,
            palette_cycle_inner,
            glyph_inner,
        );

        if args.grid {
            let grid_style = args.grid_style.parse().unwrap_or(GridStyle::Cross);
            let grid_color = hex_to_rgb(&args.grid_color).unwrap_or(RgbColor {
                r: 255,
                g: 255,
                b: 255,
            });
            let mut grid_renderer = GridRenderer::new(
                grid_style,
                args.grid_size,
                grid_color,
                args.grid_opacity,
                args.grid_adaptive,
            );
            grid_renderer.initialize(term_width, term_height);

            // Calculate average brightness for adaptive opacity
            let total_brightness: f32 = downsampled
                .cells()
                .iter()
                .map(|cell| cell.top.max(cell.bottom))
                .sum();
            let avg_brightness = if !downsampled.cells().is_empty() && max_brightness > 0.0 {
                (total_brightness / (downsampled.cells().len() as f32)) / max_brightness
            } else {
                0.0
            };

            for y in 0..term_height {
                for x in 0..term_width {
                    if grid_renderer.is_grid_position(x, y, term_width, term_height) {
                        let (on_vertical, on_horizontal) = grid_renderer.get_grid_lines(x, y);
                        let opacity = grid_renderer.calculate_opacity(
                            x,
                            y,
                            term_width,
                            term_height,
                            avg_brightness,
                        );
                        buffer.render_grid_background(
                            x,
                            y,
                            grid_color,
                            opacity,
                            on_vertical,
                            on_horizontal,
                        );
                    }
                }
            }
        }

        let frame_content = buffer.build_frame_string(args.plain_output, color_mode);
        let frame_filename = format!("{}/frame_{:03}.txt", args.frame_dir, frame_idx);
        std::fs::write(&frame_filename, frame_content)?;

        if args.verbose || frame_idx % 10 == 0 {
            eprintln!(
                "Captured frame {}/{} (sim step: {})",
                frame_idx + 1,
                args.frame_count,
                (frame_idx + 1) * args.frame_skip
            );
        }
    }

    let meta = serde_json::json!({
        "seed": args.seed,
        "preset": args.preset.map(|p| format!("{:?}", p)),
        "palette": args.palette,
        "frame_count": args.frame_count,
        "frame_skip": args.frame_skip,
        "terminal_size": {"width": term_width, "height": term_height},
        "resolution": {"width": args.resolution.width, "height": args.resolution.height},
    });

    let meta_json = serde_json::to_string_pretty(&meta)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    std::fs::write(format!("{}/meta.json", args.frame_dir), meta_json)?;

    eprintln!(
        "Done! Captured {} frames to {}",
        args.frame_count, args.frame_dir
    );

    Ok(())
}

/// Executes the "GIF Export" mode.
///
/// Runs the simulation and renders frames directly to an animated GIF file.
#[allow(clippy::incompatible_msrv)]
pub fn export_gif_mode(
    sim: &mut Simulation,
    args: &Args,
    palette: crate::cli::Palette,
) -> io::Result<()> {
    let output_path = args
        .export_gif
        .as_ref()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "GIF export path not provided"))?;
    let width = sim.width();
    let height = sim.height();

    eprintln!(
        "Exporting GIF to {} ({}x{}, {} frames @ {} fps)...",
        output_path, width, height, args.export_frames, args.export_fps
    );

    let config = args
        .to_sim_config()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    let charset = Charset::Ascii;

    let mut gif_exporter = GifExporter::new(width, height, output_path, args.export_fps)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut adaptive_brightness =
        AdaptiveBrightness::new(args.normalize_window, args.auto_normalize);

    // Temporal-color setup: enable EMA computation once before the loop.
    let temporal_strength = args.temporal_color;
    let temporal_mode = match args.temporal_mode.to_ascii_lowercase().as_str() {
        "accent" => crate::render::palette::TemporalMode::Accent,
        _ => crate::render::palette::TemporalMode::Hue,
    };
    if temporal_strength > 0.0 {
        let temporal_alpha = if args.temporal_lag > 0.0 {
            1.0 / args.temporal_lag
        } else {
            1.0
        };
        sim.set_compute_temporal(true, temporal_alpha);
    }
    if args.afterglow > 0.0 {
        sim.set_compute_afterglow(true, args.afterglow_rate);
    }

    let frame_skip = args.frame_skip.max(1);
    let mut downsampled_frame = DownsampledFrame::new(width, height);
    let mut blended_trail = Vec::new();
    // Reused aux frame for temporal diff downsampling.
    let mut aux_storage = crate::render::downsample::AuxFrame {
        width: 0,
        height: 0,
        cells: Vec::new(),
    };

    for frame_idx in 0..args.export_frames {
        for _ in 0..frame_skip {
            sim.update(1.0);
        }

        let sim_width = sim.width();
        let sim_height = sim.height();
        let term_width = width;
        let term_height = height;
        sim.trail_map_blended(&mut blended_trail);
        fold_afterglow(&mut blended_trail, sim.afterglow_lag(), args.afterglow);
        downsample(
            &blended_trail,
            sim_width,
            sim_height,
            term_width,
            term_height,
            &mut downsampled_frame,
        );

        adaptive_brightness.update(downsampled_frame.cells());
        let max_brightness = if args.auto_normalize {
            adaptive_brightness.get_max_brightness()
        } else {
            config.max_brightness
        };

        let species_rgb_colors = if args.species_colors_enabled() {
            Some(extract_species_rgb_colors(&config))
        } else {
            None
        };

        let background_color = config.background_color.as_ref().and_then(|c| hex_to_rgb(c));
        let intensity_mapping = args
            .to_render_art_defaults()
            .ok()
            .map(|a| a.intensity_mapping);
        let palette_cycle_gif = args
            .to_render_art_defaults()
            .ok()
            .map(|a| a.palette_cycle)
            .unwrap_or_default();

        let opt_aux_frame = if temporal_strength > 0.0 {
            crate::render::downsample::downsample_aux(
                None,
                None,
                None,
                sim.temporal_diff(),
                sim_width,
                sim_height,
                term_width,
                term_height,
                &mut aux_storage,
            );
            Some(&aux_storage)
        } else {
            None
        };

        let buffer = FrameBuffer::from_downsampled(
            downsampled_frame.cells(),
            term_width,
            term_height,
            max_brightness,
            palette.clone(),
            charset.clone(),
            args.reverse_palette,
            args.invert_palette,
            ColorMode::TrueColor,
            0.0,
            args.dither_mode().unwrap_or(DitherMode::None),
            &mut None,
            intensity_mapping.as_ref(),
            args.species_colors_enabled(),
            species_rgb_colors,
            background_color,
            args.ascii_contrast,
            opt_aux_frame,
            false,
            false,
            60.0,
            1.0,
            0.5,
            false,
            0.3,
            crate::config_defaults::TrailAgeMode::Bidirectional,
            false,
            temporal_strength,
            temporal_mode,
            palette_cycle_gif,
            crate::render::charset::GlyphConfig::default(),
        );

        let pixels = buffer.get_rgb_pixels();
        gif_exporter.add_frame_rgb(&pixels);

        if args.verbose || frame_idx % 10 == 0 || frame_idx + 1 == args.export_frames {
            eprintln!(
                "Frame {}/{} (sim step: {})",
                frame_idx + 1,
                args.export_frames,
                (frame_idx + 1) * frame_skip
            );
        }
    }

    gif_exporter
        .finish(output_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    eprintln!(
        "Done! Exported {} frames to {}",
        args.export_frames, output_path
    );

    Ok(())
}

/// Executes the "WebM Export" mode.
///
/// Runs the simulation and streams frames to an external FFmpeg process
/// to generate a WebM video file. Requires FFmpeg to be installed.
#[allow(clippy::incompatible_msrv)]
pub fn export_webm_mode(
    sim: &mut Simulation,
    args: &Args,
    palette: cli::Palette,
) -> io::Result<()> {
    let output_path = args
        .export_webm
        .as_ref()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "WebM export path not provided"))?;
    let width = sim.width();
    let height = sim.height();

    eprintln!(
        "Exporting WebM to {} ({}x{}, {} frames @ {} fps)...",
        output_path, width, height, args.export_frames, args.export_fps
    );
    eprintln!("Note: Requires FFmpeg to be installed with libvpx-vp9 encoder");

    let config = args
        .to_sim_config()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let mut webm_exporter = WebmExporter::new(width, height, output_path, args.export_fps)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let mut adaptive_brightness =
        AdaptiveBrightness::new(args.normalize_window, args.auto_normalize);

    // Temporal-color setup: enable EMA computation once before the loop.
    let temporal_strength = args.temporal_color;
    let temporal_mode = match args.temporal_mode.to_ascii_lowercase().as_str() {
        "accent" => crate::render::palette::TemporalMode::Accent,
        _ => crate::render::palette::TemporalMode::Hue,
    };
    if temporal_strength > 0.0 {
        let temporal_alpha = if args.temporal_lag > 0.0 {
            1.0 / args.temporal_lag
        } else {
            1.0
        };
        sim.set_compute_temporal(true, temporal_alpha);
    }
    if args.afterglow > 0.0 {
        sim.set_compute_afterglow(true, args.afterglow_rate);
    }

    let frame_skip = args.frame_skip.max(1);
    let mut downsampled_frame = DownsampledFrame::new(width, height);
    let mut blended_trail = Vec::new();
    // Reused aux frame for temporal diff downsampling.
    let mut aux_storage = crate::render::downsample::AuxFrame {
        width: 0,
        height: 0,
        cells: Vec::new(),
    };

    for frame_idx in 0..args.export_frames {
        for _ in 0..frame_skip {
            sim.update(1.0);
        }

        let sim_width = sim.width();
        let sim_height = sim.height();
        let term_width = width;
        let term_height = height;
        sim.trail_map_blended(&mut blended_trail);
        fold_afterglow(&mut blended_trail, sim.afterglow_lag(), args.afterglow);
        downsample(
            &blended_trail,
            sim_width,
            sim_height,
            term_width,
            term_height,
            &mut downsampled_frame,
        );

        adaptive_brightness.update(downsampled_frame.cells());
        let max_brightness = if args.auto_normalize {
            adaptive_brightness.get_max_brightness()
        } else {
            config.max_brightness
        };

        let species_rgb_colors = if args.species_colors_enabled() {
            Some(extract_species_rgb_colors(&config))
        } else {
            None
        };

        let background_color = config.background_color.as_ref().and_then(|c| hex_to_rgb(c));
        let intensity_mapping = args
            .to_render_art_defaults()
            .ok()
            .map(|a| a.intensity_mapping);
        let palette_cycle_webm = args
            .to_render_art_defaults()
            .ok()
            .map(|a| a.palette_cycle)
            .unwrap_or_default();

        let opt_aux_frame = if temporal_strength > 0.0 {
            crate::render::downsample::downsample_aux(
                None,
                None,
                None,
                sim.temporal_diff(),
                sim_width,
                sim_height,
                term_width,
                term_height,
                &mut aux_storage,
            );
            Some(&aux_storage)
        } else {
            None
        };

        let buffer = FrameBuffer::from_downsampled(
            downsampled_frame.cells(),
            term_width,
            term_height,
            max_brightness,
            palette.clone(),
            Charset::Ascii,
            args.reverse_palette,
            args.invert_palette,
            ColorMode::TrueColor,
            0.0,
            args.dither_mode().unwrap_or(DitherMode::None),
            &mut None,
            intensity_mapping.as_ref(),
            args.species_colors_enabled(),
            species_rgb_colors,
            background_color,
            args.ascii_contrast,
            opt_aux_frame,
            false,
            false,
            60.0,
            1.0,
            0.5,
            false,
            0.3,
            crate::config_defaults::TrailAgeMode::Bidirectional,
            false,
            temporal_strength,
            temporal_mode,
            palette_cycle_webm,
            crate::render::charset::GlyphConfig::default(),
        );

        let pixels = buffer.get_rgb_pixels();
        webm_exporter
            .add_frame_png(&pixels)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        if args.verbose || frame_idx % 10 == 0 || frame_idx + 1 == args.export_frames {
            eprintln!(
                "Frame {}/{} (sim step: {})",
                frame_idx + 1,
                args.export_frames,
                (frame_idx + 1) * frame_skip
            );
        }
    }

    webm_exporter
        .finish(output_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    eprintln!(
        "Done! Exported {} frames to {}",
        args.export_frames, output_path
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{ColorMode, PauseStyle};
    use crate::render::palette::{IntensityMapping, Palette, PaletteCycle, PaletteCycleMode};
    use crate::simulation::config::WindowFrame;
    use crate::terminal::control::MouseInteractionMode;

    #[test]
    fn sync_renderer_caches_pushes_charset_and_window_frame() {
        let mut rs = RuntimeState::new(
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
        let mut r = TerminalRenderer::new(
            80,
            24,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
            None,
        );
        let ascii_idx = ALL_CHARSETS
            .iter()
            .position(|c| *c == Charset::Ascii)
            .unwrap();
        rs.charset_index = ascii_idx;
        rs.window_frame = WindowFrame::Negative;
        sync_renderer_caches(&rs, &mut r);
        assert_eq!(r.charset(), &rs.current_charset());
        assert_eq!(r.window_frame(), rs.window_frame);
    }

    #[test]
    fn sync_renderer_caches_pushes_palette_cycle() {
        let mut rs = RuntimeState::new(
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
        let mut r = TerminalRenderer::new(
            80,
            24,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
            None,
        );
        let non_identity = PaletteCycle {
            cycles: 3,
            mode: PaletteCycleMode::Wrap,
        };
        rs.palette_cycle = non_identity;
        sync_renderer_caches(&rs, &mut r);
        assert_eq!(r.palette_cycle().cycles, 3);
        assert_eq!(r.palette_cycle().mode, PaletteCycleMode::Wrap);
    }

    #[test]
    fn sync_renderer_caches_pushes_glyph() {
        let mut rs = RuntimeState::new(
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
        let mut r = TerminalRenderer::new(
            80,
            24,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
            None,
        );
        rs.glyph = crate::render::charset::GlyphConfig {
            selection: Some(crate::render::charset::GlyphSelection::Hybrid),
            edge_threshold: 0.3,
        };
        sync_renderer_caches(&rs, &mut r);
        assert_eq!(
            r.glyph().selection,
            Some(crate::render::charset::GlyphSelection::Hybrid)
        );
        assert_eq!(r.glyph().edge_threshold, 0.3);
    }

    #[test]
    fn apply_live_params_syncs_sim_config_and_sigma() {
        let mut rs = RuntimeState::new(
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
        let mut sim = Simulation::new(400, 400, SimConfig::default(), 42, InitMode::Random, 0);
        let mut r = TerminalRenderer::new(
            80,
            24,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
            None,
        );
        rs.sensor_angle = 37.5;
        rs.diffusion_sigma = sim.trail_map().gaussian_sigma() + 2.0;
        let expected_sigma = rs.diffusion_sigma;
        apply_live_params(&rs, &mut sim, &mut r);
        assert_eq!(sim.config().sensor_angle, 37.5);
        assert_eq!(sim.trail_map().gaussian_sigma(), expected_sigma);
    }

    const HELP_LINES_TOP: &str = "┌─ tslime controls ───────────────────────┐";
    const HELP_LINES_BOTTOM: &str = "└─────────────────────────────────────────┘";
    const HELP_LINES_CONTENT: [&str; 7] = [
        "│ p: Pause/Resume                         │",
        "│ r: Restart                              │",
        "│ 1-5: Presets  (Network,Exploratory,etc) │",
        "│ +/-: Time scale (0.5x - 4.0x)           │",
        "│ c: Cycle palette (Shift+C reverse)      │",
        "│ h: Toggle this help                     │",
        "│ q: Quit                                 │",
    ];

    #[test]
    fn test_help_overlay_consistent_width() {
        let expected_width = HELP_LINES_TOP.chars().count();
        assert_eq!(expected_width, 43, "Top border should be 43 characters");

        assert_eq!(
            HELP_LINES_BOTTOM.chars().count(),
            expected_width,
            "Bottom border should be {} characters",
            expected_width
        );

        for (i, line) in HELP_LINES_CONTENT.iter().enumerate() {
            assert_eq!(
                line.chars().count(),
                expected_width,
                "Content line {} should be {} characters but is {}: '{}'",
                i,
                expected_width,
                line.chars().count(),
                line
            );
        }
    }

    #[test]
    fn test_help_overlay_border_matching() {
        assert!(
            HELP_LINES_TOP.starts_with('┌') && HELP_LINES_TOP.ends_with('┐'),
            "Top border should start with ┌ and end with ┐"
        );
        assert!(
            HELP_LINES_BOTTOM.starts_with('└') && HELP_LINES_BOTTOM.ends_with('┘'),
            "Bottom border should start with └ and end with ┘"
        );
        assert_eq!(
            HELP_LINES_TOP.chars().count(),
            HELP_LINES_BOTTOM.chars().count(),
            "Top and bottom borders should have the same length"
        );
    }

    #[test]
    fn test_help_overlay_all_content_lines_have_pipes() {
        for (i, line) in HELP_LINES_CONTENT.iter().enumerate() {
            assert!(
                line.starts_with('│') && line.ends_with('│'),
                "Content line {} should start and end with │: '{}'",
                i,
                line
            );
        }
    }

    /// Proves that temporal_strength > 0.0 reaches the export render path and
    /// changes pixel output relative to temporal_strength == 0.0.
    ///
    /// This is the seam test for B9.  It directly exercises the
    /// set_compute_temporal → downsample_aux → from_downsampled call chain that
    /// capture_frames_mode, export_gif_mode, and export_webm_mode now use.
    ///
    /// The test works at two levels:
    ///  1. Structural: verifies that set_compute_temporal allocates the diff
    ///     buffer and that temporal_diff() returns Some(...) with non-zero values
    ///     after running the simulation.
    ///  2. Pixel: synthetically injects a known non-zero signed_diff into an
    ///     AuxFrame cell and asserts that from_downsampled with a temporal
    ///     strength above 0.0 produces different pixels than with strength 0.0,
    ///     using the same DownsampledFrame so the only variable is temporal modulation.
    #[test]
    fn temporal_color_changes_export_pixels() {
        use crate::render::downsample::{AuxCell, AuxFrame, Cell as DownsampleCell};
        use crate::render::palette::TemporalMode;

        let term_width = 8;
        let term_height = 4;
        let n_cells = term_width * term_height;

        // Build a slice of cells with non-zero brightness (trail value = 50.0, max = 100.0,
        // so brightness = 0.5 — well above background).
        let cells: Vec<DownsampleCell> = (0..n_cells)
            .map(|_| DownsampleCell {
                top: 50.0,
                bottom: 50.0,
                ..Default::default()
            })
            .collect();

        // Build an AuxFrame with a large signed_diff in every cell.
        // diff_norm = signed_diff / max_brightness = 20.0 / 100.0 = 0.2
        // blend = tanh(6 * 0.2) = tanh(1.2) ≈ 0.83 — well above zero.
        let aux = AuxFrame {
            width: term_width,
            height: term_height,
            cells: (0..n_cells)
                .map(|_| AuxCell {
                    age: 0.0,
                    delta: 0.0,
                    gradient: 0.0,
                    signed_diff: 20.0, // large positive diff
                })
                .collect(),
        };

        let max_brightness = 100.0_f32;
        let call_from_downsampled =
            |temporal_strength: f32, temporal_mode: TemporalMode, opt_aux: Option<&AuxFrame>| {
                FrameBuffer::from_downsampled(
                    &cells,
                    term_width,
                    term_height,
                    max_brightness,
                    crate::cli::Palette::Organic,
                    Charset::Ascii,
                    false,
                    false,
                    ColorMode::TrueColor,
                    0.0,
                    crate::render::dither::DitherMode::None,
                    &mut None,
                    None,
                    false,
                    None,
                    None,
                    1.0,
                    opt_aux,
                    false,
                    false,
                    60.0,
                    1.0,
                    0.5,
                    false,
                    0.3,
                    crate::config_defaults::TrailAgeMode::Bidirectional,
                    false,
                    temporal_strength,
                    temporal_mode,
                    crate::render::palette::PaletteCycle::default(),
                    crate::render::charset::GlyphConfig::default(),
                )
                .get_rgb_pixels()
            };

        let pixels_off = call_from_downsampled(0.0, TemporalMode::Hue, None);
        let pixels_on_hue = call_from_downsampled(0.8, TemporalMode::Hue, Some(&aux));
        let pixels_on_accent = call_from_downsampled(0.8, TemporalMode::Accent, Some(&aux));

        assert_eq!(
            pixels_off.len(),
            pixels_on_hue.len(),
            "buffer sizes must match"
        );

        assert_ne!(
            pixels_off, pixels_on_hue,
            "temporal_strength=0.8 hue mode with signed_diff=20 must shift pixel colors"
        );
        assert_ne!(
            pixels_off, pixels_on_accent,
            "temporal_strength=0.8 accent mode with signed_diff=20 must shift pixel colors"
        );

        // Also verify the simulation side: set_compute_temporal allocates the
        // diff buffer and after a few steps it is non-zero.
        let mut sim = Simulation::new(80, 40, SimConfig::default(), 42, InitMode::Random, 0);
        assert!(
            sim.temporal_diff().is_none(),
            "temporal_diff should be None before set_compute_temporal"
        );
        sim.set_compute_temporal(true, 0.3);
        for _ in 0..5 {
            sim.update(1.0);
        }
        let diff = sim
            .temporal_diff()
            .expect("temporal_diff should be Some after set_compute_temporal");
        let max_abs_diff = diff.iter().map(|v| v.abs()).fold(0.0f32, f32::max);
        assert!(
            max_abs_diff > 0.0,
            "temporal_diff must contain at least one non-zero value after 5 sim steps"
        );
    }

    #[test]
    fn fold_afterglow_is_noop_at_zero_and_adds_at_positive() {
        let mut blended = vec![1.0_f32, 2.0, 3.0];
        let lag = [10.0_f32, 10.0, 10.0];
        let mut copy = blended.clone();
        crate::app::fold_afterglow(&mut copy, Some(&lag), 0.0);
        assert_eq!(copy, blended, "glow_mix=0 must be byte-identical");
        crate::app::fold_afterglow(&mut blended, Some(&lag), 0.5);
        assert_eq!(blended, vec![6.0, 7.0, 8.0], "adds glow_mix·lag");
    }

    #[test]
    fn set_compute_afterglow_allocates_buffer() {
        let mut sim = Simulation::new(80, 40, SimConfig::default(), 42, InitMode::Random, 0);
        assert!(sim.afterglow_lag().is_none());
        sim.set_compute_afterglow(true, 0.05);
        sim.update(1.0);
        assert!(sim.afterglow_lag().is_some());
    }
}
