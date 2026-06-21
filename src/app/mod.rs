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
    // Push the EXACT live palette/charset (incl. Custom/CustomAscii), not the lossy
    // index reconstruction — so a custom palette in a loaded config survives apply.
    renderer.set_palette(runtime_state.live_palette.clone());
    renderer.set_invert_palette(runtime_state.invert_palette);
    renderer.set_reverse_palette(runtime_state.reverse_palette);
    renderer.set_charset(runtime_state.live_charset.clone());
    renderer.set_color_aa(runtime_state.current_color_aa());
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
    // LOSSLESS wind: use the precise vector, not the coarse direction reconstruction
    // (which would rewrite e.g. (0.3, 0.0) into (1.0, 0.0)) ([P1b]).
    new_config.wind = runtime_state.wind;
    sim.update_config(new_config);

    sync_renderer_caches(runtime_state, renderer);
}

/// Apply a fully-resolved render config to the live renderer, runtime state, and
/// sim compute buffers. Shared by startup, live preset-switch, reset, and the
/// config-load apply seam so the paths can't diverge.
///
/// Sets `rs.live_palette`/`rs.live_charset` to the EXACT resolved values (incl.
/// `Custom`/`CustomAscii`) in addition to the lossy indices, so a custom palette
/// survives a load instead of falling back to the index palette ([P0] render-lossy).
pub(crate) fn apply_render_config(
    r: &crate::render_art_defaults::ResolvedRenderConfig,
    rs: &mut RuntimeState,
    renderer: &mut TerminalRenderer,
    sim: &mut Simulation,
) {
    // Exact live values (drive the renderer; survive Custom palette/charset).
    rs.live_palette = r.palette.clone();
    rs.live_charset = r.charset.clone();

    // Palette + charset indices (RuntimeState drives index; renderer drives value).
    rs.palette_index = if let cli::Palette::Custom(_) = r.palette {
        4 // Forest fallback index for custom palettes (mirror startup)
    } else {
        ALL_PALETTES
            .iter()
            .position(|p| *p == r.palette)
            .unwrap_or(4)
    };
    if let Some(i) = ALL_CHARSETS.iter().position(|c| *c == r.charset) {
        rs.charset_index = i;
    }
    rs.color_aa[rs.charset_index] = r.color_aa;
    renderer.set_color_aa(rs.current_color_aa());

    renderer.set_intensity_mapping(Some(r.intensity_mapping.clone()));
    rs.intensity_mapping = r.intensity_mapping.clone();
    rs.intensity_mapping_index = RuntimeState::find_intensity_mapping_index(&r.intensity_mapping);
    renderer.set_palette_cycle(r.palette_cycle);
    rs.palette_cycle = r.palette_cycle;
    renderer.set_glyph(r.glyph);
    rs.glyph = r.glyph;

    // Resolved hue-shift speed is authoritative: startup, live preset-switch, and
    // reset all re-resolve it here, deliberately overriding runtime key-cycling.
    // Buckets via the single source of truth shared with the dirty projection.
    rs.palette_shift_speed = crate::terminal::state::palette_shift_speed_of(r.hue_shift);

    // Temporal + afterglow runtime state and sim compute toggles.
    rs.temporal_color = r.temporal_color;
    rs.temporal_lag_frames = r.temporal_lag_frames;
    rs.temporal_mode = r.temporal_mode;
    rs.temporal_accent = r.temporal_accent;
    rs.afterglow = r.afterglow;
    rs.afterglow_rate = r.afterglow_rate;
    sim.set_compute_temporal(r.temporal_color > 0.0, r.temporal_lag_alpha());
    sim.set_compute_afterglow(r.afterglow > 0.0, r.afterglow_rate);
}

/// A fresh, per-call-unique seed for unpinned restarts.
///
/// MUST be per-call-unique ([P1]): seconds-resolution `SystemTime` would let two
/// swaps in one second reuse a seed. `rand::random::<u64>()` is unique per call.
pub(crate) fn fresh_seed() -> u64 {
    rand::random::<u64>()
}

/// Build the background grid overlay from the app-runtime config.
///
/// Mirrors the startup grid-build (all five `GridRenderer::new` params + initialize).
/// Returns `None` when the grid is disabled.
pub(crate) fn build_grid_renderer(
    app: &crate::app_config::AppRuntimeConfig,
    term_size: (usize, usize),
) -> Option<GridRenderer> {
    if !app.grid {
        return None;
    }
    let mut grid = GridRenderer::new(
        app.grid_style,
        app.grid_size,
        app.grid_color,
        app.grid_opacity,
        app.grid_adaptive,
    );
    grid.initialize(term_size.0, term_size.1);
    Some(grid)
}

/// Recompute the window layout + buffers for a new sim config, mirroring the
/// RESIZE handler (not just startup): update the cached `window`, push dimensions
/// and the recomputed layout into the renderer, and resize the runner's frame/aux
/// buffers so they stay consistent with the (possibly changed) render dimensions.
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_window(
    sim_config: &SimConfig,
    window: &mut crate::render::window::Window,
    renderer: &mut TerminalRenderer,
    downsampled_frame: &mut DownsampledFrame,
    aux_frame: &mut crate::render::downsample::AuxFrame,
    term_size: (usize, usize),
) {
    use crate::simulation::config::ChromeStyle;
    let (tw, th) = term_size;

    // Update the cached window geometry from the new sim config.
    window.aspect = sim_config.aspect;
    window.padding = sim_config.window_padding;
    window.min_sim_size = sim_config.min_sim_size;
    window.min_frame_size = sim_config.min_frame_size;

    renderer.set_dimensions(tw, th);

    // Recompute layout + derive render dims from the SAME layout (resize-block parity).
    let (render_w, render_h) = {
        let layout = if matches!(sim_config.chrome_style, ChromeStyle::Fullscreen) {
            None
        } else {
            let l = window.compute_rects(tw, th);
            if matches!(l.fallback, crate::render::window::FallbackMode::Fullscreen) {
                None
            } else {
                Some(l)
            }
        };
        let dims = layout
            .as_ref()
            .map(|l| (l.sim_w, l.sim_h))
            .unwrap_or((tw, th));
        renderer.set_window_layout(layout);
        dims
    };

    // Resize the runner's frame buffers; a stale buffer would mismatch render dims.
    if downsampled_frame.width() != render_w || downsampled_frame.height() != render_h {
        *downsampled_frame = DownsampledFrame::new(render_w, render_h);
    }
    if aux_frame.width != render_w || aux_frame.height != render_h {
        aux_frame.width = render_w;
        aux_frame.height = render_h;
        aux_frame.cells = vec![crate::render::downsample::AuxCell::default(); render_w * render_h];
    }
}

/// The ONE total apply seam: applies EVERY lever of a `ProfileOverrides` —
/// sim, render, app-runtime, and apply-only flags — and totally syncs the
/// renderer, window, grid, and (when `restart`) the simulation seed + init mode.
///
/// This is the spine of config-load: a load applies all levers, syncs the renderer
/// completely (incl. Custom palette/charset via `live_*`), preserves PRECISE wind,
/// and restarts with the correct init mode + seed (unpinned saved config → a fresh
/// per-call-unique random seed; pinned `seed` honored verbatim).
#[allow(clippy::too_many_arguments)]
pub(crate) fn apply_overrides(
    ov: &crate::profile_overrides::ProfileOverrides,
    rs: &mut RuntimeState,
    renderer: &mut TerminalRenderer,
    sim: &mut Simulation,
    timer: &mut FrameTimer,
    grid_renderer: &mut Option<GridRenderer>,
    window: &mut crate::render::window::Window,
    downsampled_frame: &mut DownsampledFrame,
    aux_frame: &mut crate::render::downsample::AuxFrame,
    term_size: (usize, usize),
    restart: bool,
) -> Result<(), String> {
    let profile = ov.resolve()?;

    // 1. Full sim push (terrain, PRECISE wind, attractors, obstacles+masks, …).
    sim.update_config(profile.sim.clone());

    // 2. Mirror sim levers into RuntimeState — LOSSLESS wind ([P1b]).
    rs.sync_sim_levers(&profile.sim);

    // 3. App-runtime config ([P0a]) — the live source for warmup/auto-reset/grid/food.
    rs.app = profile.app.clone();

    // 4. Render side: indices + intensity/cycle/glyph/temporal/afterglow + sim toggles.
    //    apply_render_config sets rs.live_palette/live_charset to EXACT values.
    apply_render_config(&profile.render, rs, renderer, sim);

    // 5. Apply-only flags the resolved Profile does NOT carry — straight from overrides.
    rs.reverse_palette = ov.reverse_palette.unwrap_or(false);
    rs.invert_palette = ov.invert_palette.unwrap_or(false);
    rs.food_persist_enabled = ov.food_persist.unwrap_or(false);
    // Per-charset color-AA: delegate to apply_color_aa_all (Task 5 helper).
    rs.apply_color_aa_all(ov);
    renderer.set_color_aa(rs.current_color_aa());

    // 6. TOTAL renderer sync ([P1a]) — push EVERYTHING the renderer caches, using
    //    the EXACT live palette/charset (not the lossy index reconstruction).
    sync_renderer_caches(rs, renderer);
    renderer.set_background_color(profile.sim.background_color.as_deref().and_then(hex_to_rgb));

    // 7. Window: route through the same recompute the resize handler uses.
    apply_window(
        &profile.sim,
        window,
        renderer,
        downsampled_frame,
        aux_frame,
        term_size,
    );
    rs.set_render_baseline(profile.render.clone());

    // 8. Rebuild the grid overlay from the new (complete) app config ([P0a]).
    *grid_renderer = build_grid_renderer(&rs.app, term_size);

    // 9. Restart with CORRECT init + seed ([P0b]).
    if restart {
        let init = profile.sim.preferred_init_mode.unwrap_or(InitMode::Food);
        let seed = profile.seed.unwrap_or_else(fresh_seed);
        rs.original_seed = seed;
        rs.original_init_mode = init;
        sim.reset(seed, init);
    }

    timer.set_time_scale(rs.time_scale);
    Ok(())
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

    if args.dump_config {
        let profile = crate::profile_overrides::ProfileOverrides::from_args(&args)
            .and_then(|o| o.resolve())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        print!(
            "{}",
            crate::profile_overrides::dump_sim_config(&profile.sim)
        );
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
    let render = crate::profile::Profile::resolve_from_args(&args)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
        .render;
    let palette = render.palette;
    let charset = render.charset;

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
    let render = crate::profile::Profile::resolve_from_args(args)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
        .render;
    let temporal_strength = render.temporal_color;
    let temporal_mode = render.temporal_mode;
    let temporal_accent = render.temporal_accent;
    if temporal_strength > 0.0 {
        let lag = render.temporal_lag_frames;
        let temporal_alpha = if lag > 0.0 { 1.0 / lag.max(1.0) } else { 1.0 };
        sim.set_compute_temporal(true, temporal_alpha);
        // Warm up enough frames so the EMA lag buffer is populated before we
        // capture the golden frame (lag_frames warmup gives a representative diff).
        let warmup = (lag.ceil() as usize).max(1);
        for _ in 0..warmup {
            sim.update(1.0);
        }
    }
    let afterglow_val = render.afterglow;
    let afterglow_rate_val = render.afterglow_rate;
    if afterglow_val > 0.0 {
        sim.set_compute_afterglow(true, afterglow_rate_val);
    }

    sim.update(1.0);

    let (term_width, term_height) = get_terminal_size();

    let sim_width = sim.width();
    let sim_height = sim.height();
    let mut blended_trail = Vec::new();
    sim.trail_map_blended(&mut blended_trail);
    fold_afterglow(&mut blended_trail, sim.afterglow_lag(), afterglow_val);
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
    let intensity_mapping = Some(render.intensity_mapping.clone());
    let palette_cycle = render.palette_cycle;
    let glyph = render.glyph;

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
        temporal_accent,
        args.resolved_color_aa(&charset),
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
    let render = crate::profile::Profile::resolve_from_args(args)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
        .render;
    let temporal_strength = render.temporal_color;
    let temporal_mode = render.temporal_mode;
    let temporal_accent = render.temporal_accent;
    if temporal_strength > 0.0 {
        let lag = render.temporal_lag_frames;
        let temporal_alpha = if lag > 0.0 { 1.0 / lag.max(1.0) } else { 1.0 };
        sim.set_compute_temporal(true, temporal_alpha);
    }
    let afterglow_val = render.afterglow;
    let afterglow_rate_val = render.afterglow_rate;
    if afterglow_val > 0.0 {
        sim.set_compute_afterglow(true, afterglow_rate_val);
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
        fold_afterglow(&mut blended_trail, sim.afterglow_lag(), afterglow_val);
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
        let intensity_mapping = Some(render.intensity_mapping.clone());
        let palette_cycle_inner = render.palette_cycle;
        let glyph_inner = render.glyph;

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
            temporal_accent,
            args.resolved_color_aa(&charset),
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
    let render = crate::profile::Profile::resolve_from_args(args)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
        .render;
    let temporal_strength = render.temporal_color;
    let temporal_mode = render.temporal_mode;
    let temporal_accent = render.temporal_accent;
    if temporal_strength > 0.0 {
        let lag = render.temporal_lag_frames;
        let temporal_alpha = if lag > 0.0 { 1.0 / lag.max(1.0) } else { 1.0 };
        sim.set_compute_temporal(true, temporal_alpha);
    }
    let afterglow_val = render.afterglow;
    let afterglow_rate_val = render.afterglow_rate;
    if afterglow_val > 0.0 {
        sim.set_compute_afterglow(true, afterglow_rate_val);
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
        fold_afterglow(&mut blended_trail, sim.afterglow_lag(), afterglow_val);
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
        let intensity_mapping = Some(render.intensity_mapping.clone());
        let palette_cycle_gif = render.palette_cycle;

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
            temporal_accent,
            args.resolved_color_aa(&charset),
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
    let render = crate::profile::Profile::resolve_from_args(args)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?
        .render;
    let temporal_strength = render.temporal_color;
    let temporal_mode = render.temporal_mode;
    let temporal_accent = render.temporal_accent;
    if temporal_strength > 0.0 {
        let lag = render.temporal_lag_frames;
        let temporal_alpha = if lag > 0.0 { 1.0 / lag.max(1.0) } else { 1.0 };
        sim.set_compute_temporal(true, temporal_alpha);
    }
    let afterglow_val = render.afterglow;
    let afterglow_rate_val = render.afterglow_rate;
    if afterglow_val > 0.0 {
        sim.set_compute_afterglow(true, afterglow_rate_val);
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
        fold_afterglow(&mut blended_trail, sim.afterglow_lag(), afterglow_val);
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
        let intensity_mapping = Some(render.intensity_mapping.clone());
        let palette_cycle_webm = render.palette_cycle;

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
            temporal_accent,
            args.resolved_color_aa(&Charset::Ascii),
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
            MouseInteractionMode::Disabled,
            3.0,
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
        // Callers keep index and the EXACT live value in sync (Step 4b); sync_renderer_caches
        // now pushes the exact live_charset so Custom/CustomAscii survive a load.
        rs.live_charset = Charset::Ascii;
        rs.window_frame = WindowFrame::Negative;
        sync_renderer_caches(&rs, &mut r);
        assert_eq!(r.charset(), &rs.live_charset);
        assert_eq!(r.window_frame(), rs.window_frame);
    }

    #[test]
    fn sync_renderer_caches_pushes_palette_cycle() {
        let mut rs = RuntimeState::new(
            42,
            InitMode::Random,
            Preset::Organic,
            MouseInteractionMode::Disabled,
            3.0,
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
            MouseInteractionMode::Disabled,
            3.0,
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
            MouseInteractionMode::Disabled,
            3.0,
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
                    None,
                    crate::render::antialiasing::AaStrength::Off,
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

    // ── apply_overrides seam (Phase C, Task 3) ──────────────────────────────────

    /// Build the doubles `apply_overrides` operates on: a runtime state, renderer,
    /// simulation, frame timer, optional grid, window, and frame/aux buffers.
    #[allow(clippy::type_complexity)]
    fn apply_doubles() -> (
        RuntimeState,
        TerminalRenderer,
        Simulation,
        FrameTimer,
        Option<GridRenderer>,
        crate::render::window::Window,
        DownsampledFrame,
        crate::render::downsample::AuxFrame,
    ) {
        let rs = RuntimeState::new(
            42,
            InitMode::Random,
            Preset::Organic,
            MouseInteractionMode::Disabled,
            3.0,
            &SimConfig::default(),
            PauseStyle::Vignette,
            false,
            false,
        );
        let renderer = TerminalRenderer::new(
            80,
            24,
            Palette::Organic,
            Charset::HalfBlock,
            false,
            false,
            ColorMode::TrueColor,
            None,
        );
        let sim = Simulation::new(400, 400, SimConfig::default(), 42, InitMode::Random, 0);
        let timer = FrameTimer::with_time_scale(60, 0.0, 1.0);
        let window = crate::render::window::Window {
            aspect: SimConfig::default().aspect,
            padding: SimConfig::default().window_padding,
            min_sim_size: SimConfig::default().min_sim_size,
            min_frame_size: SimConfig::default().min_frame_size,
        };
        let downsampled = DownsampledFrame::new(80, 24);
        let aux = crate::render::downsample::AuxFrame {
            width: 80,
            height: 24,
            cells: vec![crate::render::downsample::AuxCell::default(); 80 * 24],
        };
        (rs, renderer, sim, timer, None, window, downsampled, aux)
    }

    #[test]
    fn apply_overrides_syncs_renderer_palette_charset_flags() {
        use crate::profile_overrides::ProfileOverrides;
        let (mut rs, mut renderer, mut sim, mut timer, mut grid, mut window, mut ds, mut aux) =
            apply_doubles();
        let ov = ProfileOverrides {
            palette: Some(Palette::Heat),
            charset: Some(Charset::Braille),
            reverse_palette: Some(true),
            invert_palette: Some(true),
            background_color: Some("000000".to_string()),
            ..Default::default()
        };
        crate::app::apply_overrides(
            &ov,
            &mut rs,
            &mut renderer,
            &mut sim,
            &mut timer,
            &mut grid,
            &mut window,
            &mut ds,
            &mut aux,
            (80, 24),
            false,
        )
        .expect("apply_overrides must succeed");

        assert_eq!(renderer.palette(), &Palette::Heat, "renderer palette");
        assert_eq!(renderer.charset(), &Charset::Braille, "renderer charset");
        assert!(renderer.reverse_palette(), "renderer reverse");
        assert!(renderer.invert_palette(), "renderer invert");
        assert_eq!(
            renderer.background_color(),
            Some(RgbColor { r: 0, g: 0, b: 0 }),
            "renderer background"
        );
        // Live exact values also set on runtime state.
        assert_eq!(rs.live_palette, Palette::Heat);
        assert_eq!(rs.live_charset, Charset::Braille);
    }

    #[test]
    fn apply_overrides_pushes_restart_only_sim_levers_with_precise_wind() {
        use crate::cli::WindArg;
        use crate::profile_overrides::ProfileOverrides;
        use crate::simulation::config::Wind;
        let (mut rs, mut renderer, mut sim, mut timer, mut grid, mut window, mut ds, mut aux) =
            apply_doubles();
        let ov = ProfileOverrides {
            terrain: Some("smooth".to_string()),
            wind: Some(WindArg { dx: 0.3, dy: 0.0 }),
            ..Default::default()
        };
        crate::app::apply_overrides(
            &ov,
            &mut rs,
            &mut renderer,
            &mut sim,
            &mut timer,
            &mut grid,
            &mut window,
            &mut ds,
            &mut aux,
            (80, 24),
            false,
        )
        .expect("apply_overrides must succeed");

        assert_eq!(
            sim.config().terrain,
            crate::simulation::config::TerrainType::Smooth,
            "sim terrain"
        );
        // LOSSLESS wind — preserves (0.3, 0.0), NOT the coarse East/(1.0, 0.0).
        assert_eq!(
            sim.config().wind,
            Some(Wind { dx: 0.3, dy: 0.0 }),
            "sim wind"
        );
        assert_eq!(rs.wind, Some(Wind { dx: 0.3, dy: 0.0 }), "rs.wind lossless");
    }

    #[test]
    fn apply_overrides_restart_uses_profile_init_and_fresh_seed_when_unpinned() {
        use crate::profile_overrides::ProfileOverrides;
        let (mut rs, mut renderer, mut sim, mut timer, mut grid, mut window, mut ds, mut aux) =
            apply_doubles();
        rs.original_seed = 42;
        rs.original_init_mode = InitMode::Food;

        // Unpinned: init_mode=Random, seed=None → fresh random seed, Random init.
        let ov = ProfileOverrides {
            init_mode: Some(InitMode::Random),
            seed: None,
            ..Default::default()
        };
        crate::app::apply_overrides(
            &ov,
            &mut rs,
            &mut renderer,
            &mut sim,
            &mut timer,
            &mut grid,
            &mut window,
            &mut ds,
            &mut aux,
            (80, 24),
            true,
        )
        .expect("apply_overrides must succeed");
        assert_eq!(
            rs.original_init_mode,
            InitMode::Random,
            "restart sets init from profile"
        );
        assert_ne!(rs.original_seed, 42, "unpinned restart picks a fresh seed");

        // Pinned: seed=Some(7) → original_seed honored verbatim.
        let ov_pinned = ProfileOverrides {
            init_mode: Some(InitMode::Random),
            seed: Some(7),
            ..Default::default()
        };
        crate::app::apply_overrides(
            &ov_pinned,
            &mut rs,
            &mut renderer,
            &mut sim,
            &mut timer,
            &mut grid,
            &mut window,
            &mut ds,
            &mut aux,
            (80, 24),
            true,
        )
        .expect("apply_overrides must succeed");
        assert_eq!(rs.original_seed, 7, "pinned seed honored");
    }

    #[test]
    fn apply_overrides_custom_palette_survives_apply() {
        use crate::profile_overrides::ProfileOverrides;
        let (mut rs, mut renderer, mut sim, mut timer, mut grid, mut window, mut ds, mut aux) =
            apply_doubles();
        // A custom palette in a loaded config must survive apply — the renderer should
        // show the custom palette, not the Forest index fallback.
        let custom = Palette::Custom(vec![
            RgbColor {
                r: 10,
                g: 20,
                b: 30,
            },
            RgbColor {
                r: 200,
                g: 100,
                b: 50,
            },
        ]);
        let ov = ProfileOverrides {
            palette: Some(custom.clone()),
            ..Default::default()
        };
        crate::app::apply_overrides(
            &ov,
            &mut rs,
            &mut renderer,
            &mut sim,
            &mut timer,
            &mut grid,
            &mut window,
            &mut ds,
            &mut aux,
            (80, 24),
            false,
        )
        .expect("apply_overrides must succeed");
        assert!(
            matches!(renderer.palette(), Palette::Custom(_)),
            "renderer must show the custom palette, not the Forest fallback"
        );
        assert!(matches!(rs.live_palette, Palette::Custom(_)));
    }
}
