use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use crossterm::event::Event;
use memory_stats::memory_stats;
use std::io::{self, Write};

mod cli;
mod config_manager;
mod export;
mod render;
mod simulation;
mod terminal;

use cli::{Args, ColorMode, Mode};
use export::GifExporter;
use export::WebmExporter;
use render::adaptive_brightness::AdaptiveBrightness;
use render::charset::Charset;
use render::dither::DitherMode;
use render::downsample::downsample;
use render::grid::{GridRenderer, GridStyle};
use render::options_overlay::ControlsOverlay;
use render::overlay::{
    ConfigBrowserOverlay, ConfigSaveOverlay, HelpOverlay, InfoOverlay, KeyboardHintsOverlay,
    OverlayRenderer, PresetComparisonOverlay, StatsOverlay,
};
use render::palette::{hex_to_rgb, RgbColor};
use simulation::config::{DiffusionKernel, InitMode, Preset, SimConfig, TerrainType};
use simulation::Simulation;
use terminal::control::{
    handle_key_event, num_palettes, ControlAction, MouseInteractionMode, PaletteShiftSpeed,
    RuntimeState, ALL_PALETTES,
};
use terminal::detection::{log_capabilities, TerminalCapabilities};
use terminal::input::{InputPoller, MouseEventType};
use terminal::output::FrameBuffer;
use terminal::screen::TerminalScreen;
use terminal::signal::is_shutdown_requested;
use terminal::timing::FrameTimer;

const REFERENCE_TIME_STEP: f32 = 1.0 / 30.0;

fn print_parameter_explanations() {
    println!("\n╔═══════════════════════════════════════════════════════════════════════╗");
    println!("║                    TSLIME PARAMETER REFERENCE                         ║");
    println!("╚═══════════════════════════════════════════════════════════════════════╝\n");

    println!("AGENT BEHAVIOR PARAMETERS");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  --sensor-angle <DEG> (default: 22.5)");
    println!("    Angle between left/right sensors in degrees.");
    println!("    • Smaller values (5-15°): Agents form tight, organized networks");
    println!("    • Medium values (20-30°): Balanced exploration and pattern formation");
    println!("    • Larger values (45-90°): Chaotic, exploratory behavior");
    println!("    Range: 5.0-90.0");

    println!("\n  --sensor-distance <FLOAT> (default: 9.0)");
    println!("    How far ahead agents can sense pheromones.");
    println!("    • Shorter distance (1-5): Agents follow trails closely, fine details");
    println!("    • Medium distance (6-12): Good balance of following and exploring");
    println!("    • Longer distance (15-50): Agents react to distant trails, broader patterns");
    println!("    Range: 1.0-50.0");

    println!("\n  --rotation-angle <DEG> (default: 45.0)");
    println!("    Maximum turn amount per step when changing direction.");
    println!("    • Small angles (5-20°): Smooth, flowing curves and tendrils");
    println!("    • Medium angles (30-50°): Mix of curves and sharp turns");
    println!("    • Large angles (60-90°): Sharp, angular patterns and quick direction changes");
    println!("    Range: 5.0-90.0");

    println!("\n  --step-size <FLOAT> (default: 1.0)");
    println!("    Distance agents move per simulation step.");
    println!("    • Slower movement (0.5-0.8): Dense, intricate patterns");
    println!("    • Normal movement (1.0): Balanced pattern development");
    println!("    • Faster movement (1.5-5.0): Loose, expansive patterns");
    println!("    Range: 0.5-5.0");

    println!("\n  --deposit <FLOAT> (default: 5.0)");
    println!("    Amount of pheromone deposited by agents per step.");
    println!("    • Low deposit (1-3): Faint trails, requires more agents to see patterns");
    println!("    • Medium deposit (4-6): Clear trails, good visibility");
    println!("    • High deposit (7-20): Very bright, intense trails");
    println!("    Range: 1.0-20.0");

    println!("\n\nTRAIL MAP PARAMETERS");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  --decay <FLOAT> (default: 0.5)");
    println!("    Trail persistence multiplier (applied each frame).");
    println!("    • Fast decay (0.5-0.7): Trails fade quickly, dynamic patterns");
    println!("    • Medium decay (0.8-0.9): Balanced trail persistence");
    println!("    • Slow decay (0.95-0.99): Trails persist long, accumulate over time");
    println!("    Range: 0.5-0.99");

    println!("\n  --diffusion-kernel <TYPE>");
    println!("    Algorithm used for pheromone spreading.");
    println!("    • mean3x3: Simple 3×3 averaging, sharp patterns");
    println!("    • gaussian: Smooth Gaussian blur, organic patterns");

    println!("\n  --diffusion-sigma <FLOAT>");
    println!("    Smoothness of Gaussian blur (only for gaussian kernel).");
    println!("    • Lower sigma (0.5-0.8): Less spreading, sharper details");
    println!("    • Higher sigma (1.0-2.0): More spreading, softer, blurred trails");
    println!("    Range: 0.5-2.0");

    println!("\n  --max-brightness <FLOAT> (default: 100.0)");
    println!("    Fixed maximum brightness for normalization.");
    println!("    • Lower values (1-10): High contrast, prevents flickering");
    println!("    • Medium values (20-50): Balanced brightness");
    println!("    • Higher values (75-100): More dynamic range, may flicker");
    println!("    Range: 1.0-100.0");

    println!("\n\nPOPULATION & INITIALIZATION");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  -n, --population <INT> (default: 50000)");
    println!("    Number of agents in the simulation.");
    println!("    • Small population (1k-20k): Sparse, individual trails visible");
    println!("    • Medium population (30k-70k): Good pattern density");
    println!("    • Large population (80k-200k): Dense, complex patterns");
    println!("    Range: 1000-200000");

    println!("\n  --init <MODE> (default: food)");
    println!("    Agent initialization pattern.");
    println!("    • random: Scattered throughout canvas");
    println!("    • central: Burst from center point");
    println!("    • circle: Ring around center");
    println!("    • gradient: Linear gradient distribution");
    println!("    • wave: Wave front from edge");
    println!("    • spiral: Spiral distribution");
    println!("    • clusters: Multiple random clusters");
    println!("    • food: Load from image (see --food)");

    println!("\n  --species <SPEC>");
    println!("    Define multiple species with different behaviors.");
    println!("    Format: 'name:count@sensor_angle,rotation_angle,step_size,deposit:color'");
    println!(
        "    Example: --species 'red:20k@22.5,45,1.0,5.0:ff0000,blue:30k@30,60,1.5,3.0:0000ff'"
    );
    println!("    Enables multi-species simulations with distinct movement patterns.");

    println!("\n\nENVIRONMENTAL FORCES");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  --attract <X,Y,STRENGTH>");
    println!("    Add point attractors that pull/push agents.");
    println!("    • Positive strength: Attracts agents toward point");
    println!("    • Negative strength: Repels agents away from point");
    println!("    Example: --attract 200,200,1.0 --attract 400,300,-0.5");
    println!("    Range: -10.0 to 10.0 for strength");

    println!("\n  --wind <DX,DY>");
    println!("    Apply constant wind force.");
    println!("    Example: --wind 0.5,0.0 (rightward wind)");
    println!("    Each component range: -1.0 to 1.0");

    println!("\n  --obstacle <TYPE:PARAMS>");
    println!("    Add obstacles that agents bounce off.");
    println!("    • circle:x,y,radius - Circular obstacle");
    println!("    • rect:x,y,width,height - Rectangular obstacle");
    println!("    • image:path,x,y,w,h,invert,threshold - Image-based obstacle");

    println!("\n  --terrain <TYPE> (default: none)");
    println!("    Organic terrain effects on agent movement.");
    println!("    • none: No terrain effects");
    println!("    • smooth: Gentle Perlin noise flow fields");
    println!("    • turbulent: Chaotic turbulent patterns");
    println!("    • mixed: Combination of smooth and turbulent");

    println!("\n  --terrain-strength <FLOAT> (default: 1.0)");
    println!("    Intensity of terrain influence on movement.");
    println!("    Range: 0.1-5.0");

    println!("\n\nRENDERING & DISPLAY");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  --palette <NAME> (default: forest)");
    println!("    Color scheme for rendering trails.");
    println!("    Available: organic, heat, ocean, mono, forest, neon, warm, vibrant,");
    println!("               legiblemono, slime, mold, fungus, swamp, moss, cosmic, ethereal");
    println!("    Or custom: \"#rrggbb,#rrggbb,...\" (2-11 colors)");

    println!("\n  --colors <MODE> (default: true)");
    println!("    Terminal color mode.");
    println!("    • 8: 8 colors (basic compatibility)");
    println!("    • 16: 16 colors");
    println!("    • 256: 256 colors");
    println!("    • true: 24-bit true color (16.7M colors)");

    println!("\n  --ascii, --braille, --quadrant");
    println!("    Character set for rendering.");
    println!("    • --ascii: ASCII characters only (widest compatibility)");
    println!("    • --braille: Braille Unicode characters (2× vertical resolution)");
    println!("    • --quadrant: Quadrant blocks (4× vertical resolution)");

    println!("\n  --resolution <WxH> (default: 400x200)");
    println!("    Internal simulation grid size.");
    println!("    • Smaller (200×100): Faster, less detail");
    println!("    • Default (400×200): Good balance");
    println!("    • Larger (800×400): Slower, more detail");

    println!("\n  --dither-mode <MODE> (default: none)");
    println!("    Dithering algorithm for color quantization.");
    println!("    • none: No dithering");
    println!("    • ordered: Bayer matrix ordered dithering");
    println!("    • error-diffusion: Floyd-Steinberg error diffusion");
    println!("    • hybrid: Combination of ordered and error diffusion");

    println!("\n\nPERFORMANCE & TIMING");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  --fps <INT> (default: 30)");
    println!("    Target frames per second for animation.");
    println!("    • Lower FPS (10-20): Slower animation, lower CPU usage");
    println!("    • Normal FPS (25-30): Smooth animation");
    println!("    • High FPS (60+): Very smooth, requires fast hardware");

    println!("\n  --time-scale <FLOAT> (default: 1.0)");
    println!("    Speed multiplier for simulation time.");
    println!("    • Slower (0.1-0.5): Slow-motion effect");
    println!("    • Normal (1.0): Real-time");
    println!("    • Faster (1.5-10.0): Fast-forward effect");
    println!("    Range: 0.1-10.0");

    println!("\n  --simd-off");
    println!("    Disable SIMD acceleration for diffusion.");
    println!("    Use scalar fallback (slower but more compatible).");

    println!("\n\nPRESETS");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  --preset <NAME>");
    println!("    Use pre-configured parameter combinations.");
    println!("    • network: Tight, organized networks with sharp edges");
    println!("    • exploratory: Chaotic, wide-ranging exploration");
    println!("    • tendrils: Long, flowing tendril-like patterns");
    println!("    • organic: Balanced, natural-looking patterns");
    println!("    • minimal: Sparse, minimalist aesthetic");
    println!("    • moss: Dense, moss-like growth patterns");
    println!("    • cosmic: Nebula-like, ethereal patterns");
    println!("    • fire: Intense, flame-like movement");
    println!("    • zen: Calm, meditative patterns");
    println!("    • storm: Turbulent, stormy patterns with wind");
    println!("    • river: Flowing, river-like patterns with directional wind");
    println!("    • ethereal: Delicate, ghostly patterns");

    println!("\n\nMODES OF OPERATION");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  -l, --live");
    println!("    Continuous animation mode (interactive).");

    println!("\n  -S, --screensaver");
    println!("    Screensaver mode - exits on any keypress.");

    println!("\n  -p, --print");
    println!("    Print a single frame and exit (non-interactive).");

    println!("\n  --export-gif <PATH>");
    println!("    Export animation to GIF file.");

    println!("\n  --export-webm <PATH>");
    println!("    Export animation to WebM video (requires FFmpeg).");

    println!("\n\nEXAMPLES");
    println!("─────────────────────────────────────────────────────────────────────────");

    println!("\n  # Fast, tight networks");
    println!("  tslime --sensor-angle 15 --rotation-angle 30 --decay 0.85");

    println!("\n  # Slow, flowing tendrils");
    println!("  tslime --sensor-angle 30 --step-size 2.0 --decay 0.90");

    println!("\n  # Chaotic exploration");
    println!("  tslime --sensor-angle 45 --rotation-angle 60 --sensor-distance 15");

    println!("\n  # Multi-species competition");
    println!("  tslime --species 'red:20k:ff0000,blue:20k:0000ff' --separate-species-trails");

    println!("\n  # Wind-driven river pattern");
    println!("  tslime --preset river --wind 0.3,0.0");

    println!("\n  # High-res export");
    println!("  tslime --resolution 800x400 --export-gif output.gif --export-frames 100");

    println!("\n\nFor more information, visit: https://github.com/yourusername/tslime");
    println!();
}

fn extract_species_rgb_colors(config: &SimConfig) -> Vec<RgbColor> {
    config
        .species_configs
        .iter()
        .filter_map(|s| hex_to_rgb(&s.color))
        .collect()
}

fn apply_random_config(
    runtime_state: &RuntimeState,
    sim: &mut Simulation,
    renderer: &mut crate::terminal::output::TerminalRenderer,
    palette_list: &[cli::Palette; 16],
    current_max_brightness: &mut f32,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Apply all new parameters to config
    let mut new_config = sim.config().clone();
    new_config.sensor_angle = runtime_state.sensor_angle;
    new_config.rotation_angle = runtime_state.turn_angle;
    new_config.step_size = runtime_state.step_size;
    new_config.decay_factor = runtime_state.decay_factor;
    new_config.deposit_amount = runtime_state.deposit_amount;
    new_config.diffusion_kernel = runtime_state.diffusion_kernel;
    new_config.wind = runtime_state.wind_direction.to_wind();
    new_config.max_brightness = runtime_state.max_brightness;
    new_config.terrain = runtime_state.terrain_type;
    new_config.terrain_strength = runtime_state.terrain_strength;

    // Randomize attractors
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

    // Randomize obstacles
    new_config.obstacles.clear();
    if rng.gen_bool(0.4) {
        let num_obstacles = rng.gen_range(1..4);
        for _ in 0..num_obstacles {
            if rng.gen_bool(0.5) {
                // Circle
                new_config
                    .obstacles
                    .push(simulation::config::Obstacle::Circle {
                        x: rng.gen_range(0.0..sim.width() as f32),
                        y: rng.gen_range(0.0..sim.height() as f32),
                        radius: rng.gen_range(10.0..40.0),
                    });
            } else {
                // Rect
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

    // Update renderer with new palette
    renderer.set_palette(runtime_state.current_palette(palette_list));

    // Also update renderer brightness target
    *current_max_brightness = runtime_state.max_brightness;
}

/// Generate shell completions and print to stdout
fn generate_completions(shell: &str) -> io::Result<()> {
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

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Handle --completions flag early
    if let Some(ref shell) = args.completions {
        generate_completions(shell)?;
        return Ok(());
    }

    // Handle --explain flag early, before any other processing
    if args.explain {
        print_parameter_explanations();
        return Ok(());
    }

    args.validate()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let config = args.to_sim_config();
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

    let mut sim = Simulation::new(
        args.resolution.width,
        args.resolution.height,
        config,
        seed,
        args.init,
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

fn print_mode(
    sim: &mut Simulation,
    args: &Args,
    palette: cli::Palette,
    charset: Charset,
) -> io::Result<()> {
    #[cfg(windows)]
    let _enable = enable_ansi_support::enable_ansi_support();

    sim.update(1.0);

    let (term_width, term_height) = get_terminal_size();

    let blended_trail = sim.trail_map_blended();
    let downsampled = downsample(
        &blended_trail,
        sim.width(),
        sim.height(),
        term_width,
        term_height,
    );

    let config = args.to_sim_config();
    let color_mode = args.color_mode().unwrap_or(ColorMode::Bits256);

    let mut adaptive_brightness =
        AdaptiveBrightness::new(args.normalize_window, args.auto_normalize);
    adaptive_brightness.update(downsampled.cells());
    let max_brightness = if args.auto_normalize {
        adaptive_brightness.get_max_brightness()
    } else {
        config.max_brightness
    };

    let species_rgb_colors = if args.species_colors {
        Some(extract_species_rgb_colors(&config))
    } else {
        None
    };

    let dither_mode = args.dither_mode().unwrap_or(DitherMode::None);

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
        args.species_colors,
        species_rgb_colors,
    );

    // Apply grid rendering if enabled
    if args.grid {
        let grid_style = GridStyle::from_str(&args.grid_style).unwrap_or(GridStyle::Cross);
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

        // Apply grid to each position
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

fn capture_frames_mode(
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

    let config = args.to_sim_config();
    let color_mode = args.color_mode().unwrap_or(ColorMode::Bits256);

    let mut adaptive_brightness =
        AdaptiveBrightness::new(args.normalize_window, args.auto_normalize);

    for frame_idx in 0..args.frame_count {
        for _ in 0..args.frame_skip {
            sim.update(1.0);
        }

        let blended_trail = sim.trail_map_blended();
        let downsampled = downsample(
            &blended_trail,
            sim.width(),
            sim.height(),
            term_width,
            term_height,
        );

        adaptive_brightness.update(downsampled.cells());
        let max_brightness = if args.auto_normalize {
            adaptive_brightness.get_max_brightness()
        } else {
            config.max_brightness
        };

        let species_rgb_colors = if args.species_colors {
            Some(extract_species_rgb_colors(&config))
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
            args.species_colors,
            species_rgb_colors,
        );

        // Apply grid rendering if enabled
        if args.grid {
            let grid_style = GridStyle::from_str(&args.grid_style).unwrap_or(GridStyle::Cross);
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

            // Apply grid to each position
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

fn export_gif_mode(sim: &mut Simulation, args: &Args, palette: cli::Palette) -> io::Result<()> {
    let output_path = args
        .export_gif
        .as_ref()
        .expect("export_gif_mode called without export_gif path");
    let width = sim.width();
    let height = sim.height();

    eprintln!(
        "Exporting GIF to {} ({}x{}, {} frames @ {} fps)...",
        output_path, width, height, args.export_frames, args.export_fps
    );

    let config = args.to_sim_config();
    let charset = Charset::Ascii;

    let mut gif_exporter =
        GifExporter::new(width, height, output_path, args.export_fps).map_err(io::Error::other)?;

    let mut adaptive_brightness =
        AdaptiveBrightness::new(args.normalize_window, args.auto_normalize);

    let frame_skip = args.frame_skip.max(1);

    for frame_idx in 0..args.export_frames {
        for _ in 0..frame_skip {
            sim.update(1.0);
        }

        let blended_trail = sim.trail_map_blended();
        let term_width = width;
        let term_height = height;
        let downsampled = downsample(
            &blended_trail,
            sim.width(),
            sim.height(),
            term_width,
            term_height,
        );

        adaptive_brightness.update(downsampled.cells());
        let max_brightness = if args.auto_normalize {
            adaptive_brightness.get_max_brightness()
        } else {
            config.max_brightness
        };

        let species_rgb_colors = if args.species_colors {
            Some(extract_species_rgb_colors(&config))
        } else {
            None
        };

        let buffer = FrameBuffer::from_downsampled(
            downsampled.cells(),
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
            args.species_colors,
            species_rgb_colors,
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

    gif_exporter.finish(output_path).map_err(io::Error::other)?;

    eprintln!(
        "Done! Exported {} frames to {}",
        args.export_frames, output_path
    );

    Ok(())
}

fn export_webm_mode(sim: &mut Simulation, args: &Args, palette: cli::Palette) -> io::Result<()> {
    let output_path = args
        .export_webm
        .as_ref()
        .expect("export_webm_mode called without export_webm path");
    let width = sim.width();
    let height = sim.height();

    eprintln!(
        "Exporting WebM to {} ({}x{}, {} frames @ {} fps)...",
        output_path, width, height, args.export_frames, args.export_fps
    );
    eprintln!("Note: Requires FFmpeg to be installed with libvpx-vp9 encoder");

    let config = args.to_sim_config();

    let mut webm_exporter =
        WebmExporter::new(width, height, output_path, args.export_fps).map_err(io::Error::other)?;

    let mut adaptive_brightness =
        AdaptiveBrightness::new(args.normalize_window, args.auto_normalize);

    let frame_skip = args.frame_skip.max(1);

    for frame_idx in 0..args.export_frames {
        for _ in 0..frame_skip {
            sim.update(1.0);
        }

        let blended_trail = sim.trail_map_blended();
        let term_width = width;
        let term_height = height;
        let downsampled = downsample(
            &blended_trail,
            sim.width(),
            sim.height(),
            term_width,
            term_height,
        );

        adaptive_brightness.update(downsampled.cells());
        let max_brightness = if args.auto_normalize {
            adaptive_brightness.get_max_brightness()
        } else {
            config.max_brightness
        };

        let species_rgb_colors = if args.species_colors {
            Some(extract_species_rgb_colors(&config))
        } else {
            None
        };

        let buffer = FrameBuffer::from_downsampled(
            downsampled.cells(),
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
            args.species_colors,
            species_rgb_colors,
        );

        let pixels = buffer.get_rgb_pixels();
        webm_exporter
            .add_frame_png(&pixels)
            .map_err(io::Error::other)?;

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
        .map_err(io::Error::other)?;

    eprintln!(
        "Done! Exported {} frames to {}",
        args.export_frames, output_path
    );

    Ok(())
}

#[allow(unused_assignments)]
fn run_simulation(
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
        if let Err(e) = terminal::enable_mouse_tracking() {
            eprintln!(
                "Warning: Failed to enable mouse tracking: {}. Mouse interaction disabled.",
                e
            );
        }
    }

    let color_mode = capabilities.auto_select_color_mode(args.color_mode().ok());

    let mut renderer = crate::terminal::output::TerminalRenderer::new(
        0,
        0,
        palette,
        charset.clone(),
        args.reverse_palette,
        args.invert_palette,
        color_mode,
    );
    let dither_mode = args.dither_mode().unwrap_or(DitherMode::None);
    renderer.set_dither_mode(dither_mode);
    let mut timer = FrameTimer::with_time_scale(args.fps, args.frame_delay, args.time_scale);
    timer.set_adaptive_fps(!args.no_auto_fps);
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
    let initial_palette = args.palette().unwrap_or(cli::Palette::Forest);
    let palette_list = [
        cli::Palette::Organic,
        cli::Palette::Heat,
        cli::Palette::Ocean,
        cli::Palette::Mono,
        cli::Palette::Forest,
        cli::Palette::Neon,
        cli::Palette::Warm,
        cli::Palette::Vibrant,
        cli::Palette::LegibleMono,
        cli::Palette::Slime,
        cli::Palette::Mold,
        cli::Palette::Fungus,
        cli::Palette::Swamp,
        cli::Palette::Moss,
        cli::Palette::Cosmic,
        cli::Palette::Ethereal,
    ];
    let initial_palette_index = if let cli::Palette::Custom(_) = initial_palette {
        4 // Default to Forest for custom palettes
    } else {
        palette_list
            .iter()
            .position(|p| *p == initial_palette)
            .unwrap_or(4)
    };

    let mode = args.mode();
    let show_help_by_default = !matches!(mode, cli::Mode::Screensaver);

    let mut runtime_state = RuntimeState::new(
        seed,
        args.init,
        initial_preset,
        initial_palette_index,
        show_help_by_default,
        mouse_mode,
        args.mouse_timeout,
    );
    runtime_state.dither_mode = dither_mode;
    runtime_state.show_stats = args.stats;
    renderer.set_dither_mode(dither_mode);

    // Initialize food persistence
    if args.food_persist && args.init == InitMode::Food {
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

    let config = args.to_sim_config();
    if args.species_colors {
        let species_rgb_colors = extract_species_rgb_colors(&config);
        renderer.set_species_colors(true, species_rgb_colors);
    }

    let mut adaptive_brightness =
        AdaptiveBrightness::new(args.normalize_window, args.auto_normalize);
    let mut hue_offset: f32 = 0.0;

    let mut current_auto_normalize = args.auto_normalize;
    let mut _current_max_brightness = args.max_brightness;

    // Apply initial randomization if requested
    if args.random {
        runtime_state.randomize_params();
        apply_random_config(
            &runtime_state,
            sim,
            &mut renderer,
            &palette_list,
            &mut _current_max_brightness,
        );
    }

    let start_time = std::time::Instant::now();

    // Initialize grid renderer if enabled
    let mut grid_renderer = if args.grid {
        let grid_style = GridStyle::from_str(&args.grid_style).unwrap_or(GridStyle::Cross);
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
            if new_width != term_width || new_height != term_height {
                term_width = new_width;
                term_height = new_height;
                renderer.set_dimensions(term_width as usize, term_height as usize);
                // Reinitialize grid with new dimensions
                if let Some(ref mut grid) = grid_renderer {
                    grid.initialize(term_width as usize, term_height as usize);
                }
            }
        }

        let dt = timer.delta_time();

        // Check if we're in warmup phase
        let in_warmup = !args.skip_warmup && runtime_state.is_in_warmup(args.warmup_frames);

        if !runtime_state.is_paused {
            timer.start_sim();

            if in_warmup {
                // During warmup: slow agent movement, high decay, bright display
                let warmup_dt = dt * 0.3; // 30% speed
                sim.update(warmup_dt / REFERENCE_TIME_STEP);
                runtime_state.increment_warmup();
            } else {
                sim.update(dt / REFERENCE_TIME_STEP);
            }

            timer.end_sim_start_render();
        } else {
            timer.start_sim();
            timer.end_sim_start_render();
        }

        let blended_trail = sim.trail_map_blended();
        let downsampled = downsample(
            &blended_trail,
            sim.width(),
            sim.height(),
            term_width as usize,
            term_height as usize,
        );

        let current_config = args.to_sim_config();

        adaptive_brightness.update(downsampled.cells());
        let mut max_brightness = if args.auto_normalize {
            adaptive_brightness.get_max_brightness()
        } else {
            current_config.max_brightness
        };

        // Apply warmup brightness multiplier with smooth transition
        if in_warmup || runtime_state.warmup_counter < args.warmup_frames + 30 {
            // Calculate fade factor: 1.0 during warmup, then fade to 0.0 over 30 frames
            let fade_factor = if in_warmup {
                1.0
            } else {
                let frames_since_warmup = runtime_state.warmup_counter - args.warmup_frames;
                (1.0 - (frames_since_warmup as f32 / 30.0)).max(0.0)
            };

            // Interpolate between normal and warmup brightness
            let multiplier = 1.0 + (args.warmup_brightness_multiplier - 1.0) * fade_factor;
            max_brightness *= multiplier;
        }

        let current_palette = runtime_state.current_palette(&palette_list);

        let shift_degrees = runtime_state.palette_shift_speed.degrees_per_second();
        hue_offset += shift_degrees * dt;
        hue_offset %= 360.0;
        renderer.set_hue_shift(hue_offset);

        // Build help overlay (? key)
        let help_lines: Option<Vec<String>> = if runtime_state.show_help {
            let base_help_strings = HelpOverlay::build_overlay();
            let base_help_len = base_help_strings.len();
            let base_help: Vec<&str> = base_help_strings.iter().map(|s| s.as_str()).collect();
            let attractor_lines =
                OverlayRenderer::build_help_with_attractors(&base_help, &sim.config().attractors);
            let obstacle_lines =
                OverlayRenderer::build_help_with_obstacles(&base_help, &sim.config().obstacles);
            let mouse_attractor_lines = OverlayRenderer::build_help_with_mouse_attractors(
                &base_help,
                &sim.config().mouse_attractors,
                sim.width(),
                sim.height(),
            );

            let mut result = base_help_strings;

            if !attractor_lines.is_empty() {
                // Skip the base help duplicate and add just the attractor section
                result.extend(attractor_lines.into_iter().skip(base_help_len));
            }

            if !obstacle_lines.is_empty() {
                // Skip the base help duplicate and add just the obstacle section
                result.extend(obstacle_lines.into_iter().skip(base_help_len));
            }

            if !mouse_attractor_lines.is_empty() {
                // Skip the base help duplicate and add just the mouse attractor section
                result.extend(mouse_attractor_lines.into_iter().skip(base_help_len));
            }

            Some(result)
        } else {
            None
        };

        // Build keyboard hints overlay (? key)
        let keyboard_hints_lines: Option<Vec<String>> = if runtime_state.show_keyboard_hints {
            Some(KeyboardHintsOverlay::build_overlay())
        } else {
            None
        };
        let (keyboard_hints_x, keyboard_hints_y) = if keyboard_hints_lines.is_some() {
            KeyboardHintsOverlay::calculate_position(term_width as usize, term_height as usize)
        } else {
            (0, 0)
        };

        // Build preset comparison overlay (Shift+1-7 keys)
        let preset_comparison_lines: Option<Vec<String>> = if runtime_state.show_preset_comparison {
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

        // Calculate controls Y position (below help if help is visible)
        let controls_y = if let Some(ref help) = help_lines {
            2 + help.len() + 1
        } else {
            2
        };

        // Build controls overlay (h key)
        let controls_lines: Option<Vec<String>> = if runtime_state.show_controls {
            Some(ControlsOverlay::build_overlay(
                runtime_state.controls_category_idx,
                runtime_state.sensor_angle,
                runtime_state.sensor_distance,
                runtime_state.turn_angle,
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
                runtime_state.palette_shift_speed,
                runtime_state.invert_palette,
                runtime_state.reverse_palette,
                runtime_state.dither_mode.name(),
                term_width as usize,
                runtime_state.default_values,
                sim.agent_count(),
            ))
        } else {
            None
        };

        // Build status line (shown when any overlay visible or paused)
        let diffusion_kernel_name = match runtime_state.diffusion_kernel {
            simulation::config::DiffusionKernel::Mean3x3 => "Mean3x3",
            simulation::config::DiffusionKernel::Gaussian => "Gaussian",
        };
        let status_line = render::overlay::OverlayRenderer::build_status_line(
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
        );
        let status_x =
            render::overlay::OverlayRenderer::status_line_x(&status_line, term_width as usize);
        let status_data = if runtime_state.any_overlay_open() || runtime_state.is_paused {
            Some((status_line, status_x))
        } else {
            None
        };

        // Notification at bottom center (or warmup message during warmup)
        let notification_data = if in_warmup {
            // Show warmup message during warmup phase
            let progress = (runtime_state.warmup_counter as f32 / 30.0 * std::f32::consts::PI)
                .sin()
                .abs();
            let opacity = (progress * 3.0) as usize;
            let dots = ".".repeat(opacity.min(3));
            let warmup_text = format!("[ Press any key to begin{} ]", dots);
            let warmup_x = if warmup_text.len() < term_width as usize {
                (term_width as usize - warmup_text.len()) / 2
            } else {
                0
            };
            Some((warmup_text, warmup_x))
        } else {
            runtime_state.current_notification().map(|msg| {
                let notification_text = format!("[ {} ]", msg);
                let notification_x = if notification_text.len() < term_width as usize {
                    (term_width as usize - notification_text.len()) / 2
                } else {
                    0
                };
                (notification_text, notification_x)
            })
        };

        // Stats overlay at top-right
        let entropy = StatsOverlay::calculate_entropy(&blended_trail, 100);
        let trail_sum: f32 = blended_trail.iter().sum();
        let trail_capacity = (sim.width() * sim.height()) as f32 * 10.0;
        let trail_density = if trail_capacity > 0.0 {
            (trail_sum / trail_capacity).min(1.0)
        } else {
            0.0
        };

        runtime_state.update_history(timer.current_fps() as f32, entropy, trail_density);

        let stats_lines: Option<Vec<String>> = if runtime_state.show_stats {
            let elapsed = start_time.elapsed().as_secs_f32();
            let trail_max = blended_trail.iter().fold(0.0f32, |m, &v| v.max(m));
            let memory_mb = memory_stats()
                .map(|m| m.physical_mem as f32 / 1024.0 / 1024.0)
                .unwrap_or(0.0);
            let frame_time_ms = timer.last_frame_ms();
            let cpu_percent = (frame_time_ms / 33.333) * 100.0;

            Some(StatsOverlay::build_overlay(
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
                term_width as usize,
                &runtime_state.fps_history,
                &runtime_state.entropy_history,
                &runtime_state.density_history,
            ))
        } else {
            None
        };
        let stats_x = StatsOverlay::calculate_x_position(term_width as usize);

        // Info overlay (below stats)
        let info_lines: Option<Vec<String>> = if runtime_state.show_info {
            let init_mode_name = match args.init {
                InitMode::Random => "Random",
                InitMode::CentralBurst => "Central",
                InitMode::Circle => "Circle",
                InitMode::Gradient => "Gradient",
                InitMode::WaveFront => "Wave",
                InitMode::Spiral => "Spiral",
                InitMode::RandomClusters => "Clusters",
                InitMode::Food => "Food",
            };

            let color_mode_name = match color_mode {
                ColorMode::TrueColor => "TrueColor",
                ColorMode::Bits8 => "8",
                ColorMode::Bits16 => "16",
                ColorMode::Bits256 => "256",
            };

            let charset_name = match charset {
                Charset::HalfBlock => "HalfBlock",
                Charset::Ascii => "ASCII",
                Charset::Braille => "Braille",
                Charset::Quadrant => "Quadrant",
                Charset::CustomAscii(_) => "Custom",
            };

            let food_source = if args.init == InitMode::Food {
                Some(args.food.clone())
            } else {
                None
            };

            Some(InfoOverlay::build_overlay(
                sim.width(),
                sim.height(),
                term_width as usize,
                term_height as usize,
                init_mode_name,
                color_mode_name,
                charset_name,
                !args.simd_off,
                &food_source,
                args.warmup_frames,
                args.warmup_brightness_multiplier,
                args.warmup_decay,
                args.auto_reset,
                args.collapse_entropy_threshold,
                args.collapse_duration_frames,
            ))
        } else {
            None
        };

        let info_y = if let Some(ref stats) = stats_lines {
            2 + stats.len() + 1
        } else {
            2
        };
        let info_x = InfoOverlay::calculate_x_position(term_width as usize);

        // Config browser overlay
        let config_browser_lines: Option<Vec<String>> = if runtime_state.show_config_browser {
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
        let (config_browser_x, config_browser_y) = if config_browser_lines.is_some() {
            ConfigBrowserOverlay::calculate_position(term_width as usize, term_height as usize)
        } else {
            (0, 0)
        };

        // Config save dialog overlay
        let config_save_lines: Option<Vec<String>> = if runtime_state.show_config_save_dialog {
            Some(ConfigSaveOverlay::build_overlay(
                &runtime_state.config_save_name_input,
            ))
        } else {
            None
        };
        let (config_save_x, config_save_y) = if config_save_lines.is_some() {
            ConfigSaveOverlay::calculate_position(term_width as usize, term_height as usize)
        } else {
            (0, 0)
        };

        // Food persistence fade-out
        if runtime_state.food_persist_enabled
            && !runtime_state.is_paused
            && args.food_persist_duration > 0
        {
            runtime_state.food_persist_counter += 1;

            if runtime_state.food_persist_counter <= args.food_persist_duration {
                // Calculate fade factor using quadratic easing
                let progress =
                    runtime_state.food_persist_counter as f32 / args.food_persist_duration as f32;
                let fade_factor = (1.0 - progress).powi(2); // Quadratic fade-out

                // Update attractor strengths
                let mut new_config = sim.config().clone();
                new_config.attractors.clear();

                for attractor in &runtime_state.initial_food_attractors {
                    new_config
                        .attractors
                        .push(crate::simulation::config::Attractor::new(
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

        // Entropy-based auto-reset
        if args.auto_reset && !runtime_state.is_paused {
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
                sim.reset(new_seed, args.init);
                runtime_state.reset_collapse_counter();
                runtime_state.reset_warmup();
                runtime_state.food_persist_counter = 0; // Reset food persistence counter
                runtime_state.show_notification(format!(
                    "Simulation collapsed - restarting with seed {}",
                    new_seed
                ));
            }
        }

        if max_brightness > 0.0 {
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
                    max_brightness,
                    help_lines.as_ref().map(|v| (v.as_slice(), 2usize, 2usize)),
                    controls_lines
                        .as_ref()
                        .map(|v| (v.as_slice(), 2usize, controls_y)),
                    status_data,
                    notification_data,
                    stats_lines.as_ref().map(|v| (v.as_slice(), stats_x)),
                    info_lines.as_ref().map(|v| (v.as_slice(), info_x, info_y)),
                    grid_renderer.as_ref(),
                    config_browser_lines
                        .as_ref()
                        .map(|v| (v.as_slice(), config_browser_x, config_browser_y)),
                    config_save_lines
                        .as_ref()
                        .map(|v| (v.as_slice(), config_save_x, config_save_y)),
                    keyboard_hints_lines
                        .as_ref()
                        .map(|v| (v.as_slice(), keyboard_hints_x, keyboard_hints_y)),
                    preset_comparison_lines
                        .as_ref()
                        .map(|v| (v.as_slice(), preset_comparison_x, preset_comparison_y)),
                )?;
            } else {
                renderer.render_with_overlay(
                    downsampled.cells(),
                    max_brightness,
                    help_lines.as_ref().map(|v| (v.as_slice(), 2usize, 2usize)),
                    controls_lines
                        .as_ref()
                        .map(|v| (v.as_slice(), 2usize, controls_y)),
                    status_data,
                    notification_data,
                    stats_lines.as_ref().map(|v| (v.as_slice(), stats_x)),
                    info_lines.as_ref().map(|v| (v.as_slice(), info_x, info_y)),
                    grid_renderer.as_ref(),
                    config_browser_lines
                        .as_ref()
                        .map(|v| (v.as_slice(), config_browser_x, config_browser_y)),
                    config_save_lines
                        .as_ref()
                        .map(|v| (v.as_slice(), config_save_x, config_save_y)),
                    keyboard_hints_lines
                        .as_ref()
                        .map(|v| (v.as_slice(), keyboard_hints_x, keyboard_hints_y)),
                    preset_comparison_lines
                        .as_ref()
                        .map(|v| (v.as_slice(), preset_comparison_x, preset_comparison_y)),
                )?;
            }
        }

        timer.end_render();

        let mut should_exit = false;
        let events = input_poller.drain_all_events()?;
        for event in events {
            match event {
                Event::Key(key_event) => {
                    // Skip warmup on any key press
                    if in_warmup {
                        runtime_state.warmup_counter = args.warmup_frames; // Skip to end
                        continue; // Don't process the key event during warmup
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
                                        runtime_state.current_palette(&palette_list),
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
                                        args.init,
                                        if args.init == InitMode::Food {
                                            Some(args.food.clone())
                                        } else {
                                            None
                                        },
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
                                                    runtime_state.current_palette(&palette_list);
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

                    if InputPoller::is_exit_key(&key_event) {
                        should_exit = true;
                        break;
                    }

                    let action = handle_key_event(&key_event);
                    match action {
                        ControlAction::TogglePause => {
                            runtime_state.toggle_pause();
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
                            let new_palette = runtime_state.current_palette(&palette_list);
                            renderer.set_palette(new_palette);
                        }
                        ControlAction::CyclePaletteReverse => {
                            runtime_state.cycle_palette_reverse(num_palettes());
                            let new_palette = runtime_state.current_palette(&palette_list);
                            renderer.set_palette(new_palette);
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
                        ControlAction::ToggleHelp => {
                            runtime_state.toggle_help();
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
                            runtime_state.cycle_controls_category(true);
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
                            let at_bound = runtime_state.adjust_turn_angle(delta);
                            let mut new_config = sim.config().clone();
                            new_config.rotation_angle = runtime_state.turn_angle;
                            sim.update_config(new_config);
                            if at_bound {
                                runtime_state.show_notification(format!(
                                    "Turn angle at {}°",
                                    runtime_state.turn_angle
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
                        ControlAction::ResetToDefaults => {
                            runtime_state.reset_to_defaults();
                            let new_config = SimConfig::from(runtime_state.current_preset);
                            sim.update_config(new_config);
                            _current_max_brightness = runtime_state.max_brightness;
                            renderer.set_invert_palette(runtime_state.invert_palette);
                            renderer.set_reverse_palette(runtime_state.reverse_palette);
                            runtime_state.show_notification("Reset to defaults".to_string());
                        }
                        ControlAction::ShowOptionsOverlay => {
                            // Toggle controls overlay (same as ToggleControls)
                            runtime_state.toggle_controls();
                        }
                        ControlAction::ToggleStats => {
                            runtime_state.toggle_stats();
                            runtime_state.show_notification(format!(
                                "Stats: {}",
                                if runtime_state.show_stats {
                                    "On"
                                } else {
                                    "Off"
                                }
                            ));
                        }
                        ControlAction::ToggleInfo => {
                            runtime_state.toggle_info();
                            runtime_state.show_notification(format!(
                                "Info: {}",
                                if runtime_state.show_info { "On" } else { "Off" }
                            ));
                        }
                        ControlAction::Quit => {
                            should_exit = true;
                            break;
                        }
                        ControlAction::ShowConfigBrowser => {
                            runtime_state.show_config_browser = true;
                            runtime_state.config_browser_selected_index = 0;
                        }
                        ControlAction::ShowConfigSaveDialog => {
                            runtime_state.show_config_save_dialog = true;
                            runtime_state.config_save_name_input.clear();
                        }
                        ControlAction::ConfigBrowserUp => {
                            // Handled separately in config browser input handling
                        }
                        ControlAction::ConfigBrowserDown => {
                            // Handled separately in config browser input handling
                        }
                        ControlAction::ConfigBrowserSelect => {
                            // Handled separately in config browser input handling
                        }
                        ControlAction::ConfigBrowserDelete => {
                            // Handled separately in config browser input handling
                        }
                        ControlAction::ConfigSaveConfirm => {
                            // Handled separately in config save input handling
                        }
                        ControlAction::ConfigCancel => {
                            runtime_state.show_config_browser = false;
                            runtime_state.show_config_save_dialog = false;
                        }
                        ControlAction::RandomizeParams => {
                            runtime_state.randomize_params();
                            apply_random_config(
                                &runtime_state,
                                sim,
                                &mut renderer,
                                &palette_list,
                                &mut _current_max_brightness,
                            );

                            runtime_state.show_notification("Parameters Randomized!".to_string());
                        }
                        ControlAction::Undo => {
                            if runtime_state.undo().is_some() {
                                // Apply the undone state to simulation
                                let mut new_config = sim.config().clone();
                                new_config.sensor_angle = runtime_state.sensor_angle;
                                new_config.rotation_angle = runtime_state.turn_angle;
                                new_config.step_size = runtime_state.step_size;
                                new_config.decay_factor = runtime_state.decay_factor;
                                new_config.deposit_amount = runtime_state.deposit_amount;
                                new_config.diffusion_kernel = runtime_state.diffusion_kernel;
                                new_config.diffusion_sigma = runtime_state.diffusion_sigma;
                                new_config.max_brightness = runtime_state.max_brightness;
                                new_config.terrain = runtime_state.terrain_type;
                                new_config.terrain_strength = runtime_state.terrain_strength;
                                sim.update_config(new_config);

                                renderer.set_palette(runtime_state.current_palette(&palette_list));
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
                                new_config.rotation_angle = runtime_state.turn_angle;
                                new_config.step_size = runtime_state.step_size;
                                new_config.decay_factor = runtime_state.decay_factor;
                                new_config.deposit_amount = runtime_state.deposit_amount;
                                new_config.diffusion_kernel = runtime_state.diffusion_kernel;
                                new_config.diffusion_sigma = runtime_state.diffusion_sigma;
                                new_config.max_brightness = runtime_state.max_brightness;
                                new_config.terrain = runtime_state.terrain_type;
                                new_config.terrain_strength = runtime_state.terrain_strength;
                                sim.update_config(new_config);

                                renderer.set_palette(runtime_state.current_palette(&palette_list));
                                renderer.set_invert_palette(runtime_state.invert_palette);
                                renderer.set_reverse_palette(runtime_state.reverse_palette);
                                renderer.set_dither_mode(runtime_state.dither_mode);

                                runtime_state.show_notification("Redo successful".to_string());
                            } else {
                                runtime_state.show_notification("Nothing to redo".to_string());
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
        let _ = terminal::disable_mouse_tracking();
    }

    Ok(())
}

fn get_terminal_size() -> (usize, usize) {
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
}
