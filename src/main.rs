use clap::Parser;
use crossterm::event::Event;
use std::io::{self, Write};

mod cli;
mod export;
mod render;
mod simulation;
mod terminal;

use cli::{Args, ColorMode, Mode};
use export::GifExporter;
use render::adaptive_brightness::AdaptiveBrightness;
use render::charset::Charset;
use render::dither::DitherMode;
use render::downsample::downsample;
use render::options_overlay::OptionsOverlay;
use render::palette::{hex_to_rgb, RgbColor};
use simulation::config::{DiffusionKernel, Preset, SimConfig, TerrainType};
use simulation::Simulation;
use terminal::control::{
    handle_key_event, num_palettes, ControlAction, HelpMode, MouseInteractionMode,
    PaletteShiftSpeed, RuntimeState,
};
use terminal::input::{InputPoller, MouseEventType};
use terminal::output::FrameBuffer;
use terminal::screen::TerminalScreen;
use terminal::signal::is_shutdown_requested;
use terminal::timing::FrameTimer;

const REFERENCE_TIME_STEP: f32 = 1.0 / 30.0;

fn extract_species_rgb_colors(config: &SimConfig) -> Vec<RgbColor> {
    config
        .species_configs
        .iter()
        .filter_map(|s| hex_to_rgb(&s.color))
        .collect()
}

fn main() -> io::Result<()> {
    let args = Args::parse();

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

    let buffer = FrameBuffer::from_downsampled(
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

        let buffer = FrameBuffer::from_downsampled(
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
            args.dither_mode().unwrap_or(DitherMode::None),
            &mut None,
            args.species_colors,
            species_rgb_colors,
        );

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

    std::fs::write(
        format!("{}/meta.json", args.frame_dir),
        serde_json::to_string_pretty(&meta).unwrap(),
    )?;

    eprintln!(
        "Done! Captured {} frames to {}",
        args.frame_count, args.frame_dir
    );

    Ok(())
}

fn export_gif_mode(sim: &mut Simulation, args: &Args, palette: cli::Palette) -> io::Result<()> {
    let output_path = args.export_gif.as_ref().unwrap();
    let width = sim.width();
    let height = sim.height();

    eprintln!(
        "Exporting GIF to {} ({}x{}, {} frames @ {} fps)...",
        output_path, width, height, args.export_frames, args.export_fps
    );

    let config = args.to_sim_config();
    let charset = Charset::Ascii;

    let mut gif_exporter = GifExporter::new(width, height, output_path, args.export_fps)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

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
            charset,
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

    gif_exporter
        .finish(output_path)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

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

    let mouse_mode = if args.mouse_attract {
        MouseInteractionMode::Attract
    } else if args.mouse_repel {
        MouseInteractionMode::Repel
    } else {
        MouseInteractionMode::Disabled
    };

    if mouse_mode != MouseInteractionMode::Disabled {
        if let Err(e) = terminal::enable_mouse_tracking() {
            eprintln!(
                "Warning: Failed to enable mouse tracking: {}. Mouse interaction disabled.",
                e
            );
        }
    }

    let color_mode = args.color_mode().unwrap_or(ColorMode::Bits256);

    let mut renderer = crate::terminal::output::TerminalRenderer::new(
        0,
        0,
        palette,
        charset,
        args.reverse_palette,
        args.invert_palette,
        color_mode,
    );
    let dither_mode = args.dither_mode().unwrap_or(DitherMode::None);
    renderer.set_dither_mode(dither_mode);
    let mut timer = FrameTimer::with_time_scale(args.fps, args.frame_delay, args.time_scale);
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
    ];
    let initial_palette_index = palette_list
        .iter()
        .position(|p| *p == initial_palette)
        .unwrap_or(4);

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
    renderer.set_dither_mode(dither_mode);

    let config = args.to_sim_config();
    if args.species_colors {
        let species_rgb_colors = extract_species_rgb_colors(&config);
        renderer.set_species_colors(true, species_rgb_colors);
    }

    let mut adaptive_brightness =
        AdaptiveBrightness::new(args.normalize_window, args.auto_normalize);
    let mut hue_offset: f32 = 0.0;

    let mut current_auto_normalize = args.auto_normalize;
    let mut current_max_brightness = args.max_brightness;

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
            }
        }

        let dt = timer.delta_time();

        if !runtime_state.is_paused {
            timer.start_sim();
            sim.update(dt / REFERENCE_TIME_STEP);
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
        let max_brightness = if args.auto_normalize {
            adaptive_brightness.get_max_brightness()
        } else {
            current_config.max_brightness
        };

        let current_palette = runtime_state.current_palette(&palette_list);

        let shift_degrees = runtime_state.palette_shift_speed.degrees_per_second();
        hue_offset += shift_degrees * dt;
        hue_offset %= 360.0;
        renderer.set_hue_shift(hue_offset);

        let mouse_help = if runtime_state.mouse_mode != MouseInteractionMode::Disabled {
            let mode_str = match runtime_state.mouse_mode {
                MouseInteractionMode::Attract => "attract",
                MouseInteractionMode::Repel => "repel",
                MouseInteractionMode::Disabled => "",
            };
            format!(
                "│ Click: {} agents ({:.1}s timeout)         │",
                mode_str, runtime_state.mouse_timeout
            )
        } else {
            String::new()
        };

        static HELP_LINES: [&str; 12] = [
            "┌─ tslime controls ───────────────────────┐",
            "│ p: Pause/Resume                         │",
            "│ r: Restart                              │",
            "│ 1-5: Presets  (Network,Exploratory,etc) │",
            "│ +/-: Time scale (0.5x - 4.0x)           │",
            "│ c: Cycle palette (Shift+C reverse)      │",
            "│ d: Toggle dithering                    │",
            "│ m: Cycle dither mode                   │",
            "│ []: Adjust dither intensity (0.0-1.0)   │",
            "│ h: Toggle this help                     │",
            "│ q: Quit                                 │",
            "└─────────────────────────────────────────┘",
        ];

        static QUICK_HELP_LINES: [&str; 12] = [
            "┌─ tslime controls ───────────────────────┐",
            "│ p: Pause/Resume                         │",
            "│ r: Restart                              │",
            "│ +/-: Time scale                         │",
            "│ c: Cycle palette                        │",
            "│ h: Toggle help (Tab for options)        │",
            "│ q: Quit                                 │",
            "│                                        │",
            "│ SIMULATION (A,T,S,E,I)                  │",
            "│ ENVIRONMENT (K,W,Y,U)                   │",
            "│ VISUAL (B,V,N)                          │",
            "└─────────────────────────────────────────┘",
        ];

        let help_lines = match runtime_state.help_mode {
            HelpMode::None => None,
            HelpMode::Quick => {
                let attractor_lines = render::overlay::OverlayRenderer::build_help_with_attractors(
                    &HELP_LINES,
                    &sim.config().attractors,
                );
                let obstacle_lines = render::overlay::OverlayRenderer::build_help_with_obstacles(
                    &[],
                    &sim.config().obstacles,
                );
                let mouse_attractor_lines =
                    render::overlay::OverlayRenderer::build_help_with_mouse_attractors(
                        &[],
                        &sim.config().mouse_attractors,
                        sim.width(),
                        sim.height(),
                    );

                let mut result = if obstacle_lines.is_empty() {
                    attractor_lines
                } else {
                    let mut combined = attractor_lines;
                    combined.extend(obstacle_lines);
                    combined
                };

                if !mouse_attractor_lines.is_empty() {
                    result.extend(mouse_attractor_lines);
                }

                if !mouse_help.is_empty() {
                    result.push(String::new());
                    result.push("┌─ Mouse Interaction ─────────────────────┐".to_string());
                    result.push(mouse_help);
                    result.push("└─────────────────────────────────────────┘".to_string());
                }

                Some(result)
            }
            HelpMode::Options => {
                let options_overlay = OptionsOverlay::build_overlay(
                    runtime_state.options_category_idx,
                    runtime_state.sensor_angle,
                    runtime_state.turn_angle,
                    runtime_state.step_size,
                    runtime_state.decay_factor,
                    runtime_state.deposit_amount,
                    runtime_state.diffusion_kernel,
                    runtime_state.wind_direction,
                    runtime_state.terrain_type,
                    runtime_state.terrain_strength,
                    runtime_state.auto_normalize,
                    runtime_state.motion_blur_frames,
                    runtime_state.max_brightness,
                    runtime_state.fast_mode_enabled,
                    runtime_state.palette_shift_speed,
                    runtime_state.invert_palette,
                    runtime_state.reverse_palette,
                    term_width as usize,
                );
                Some(options_overlay)
            }
        };

        let status_line = render::overlay::OverlayRenderer::build_status_line(
            runtime_state.is_paused,
            runtime_state.current_preset,
            runtime_state.time_scale,
            current_palette.clone(),
            runtime_state.dither_mode,
            term_width as usize,
        );
        let status_x =
            render::overlay::OverlayRenderer::status_line_x(&status_line, term_width as usize);
        let status_data = if runtime_state.show_help || runtime_state.is_paused {
            Some((status_line, status_x))
        } else {
            None
        };

        let paused_text = "[ PAUSED ]";
        let paused_x = render::overlay::OverlayRenderer::paused_overlay_x(term_width as usize);
        let paused_data = if runtime_state.is_paused {
            Some((paused_text.to_string(), paused_x))
        } else {
            None
        };

        let notification_data = runtime_state.current_notification().map(|msg| {
            let notification_text = format!("[ {} ]", msg);
            let notification_x = if notification_text.len() < term_width as usize {
                (term_width as usize - notification_text.len()) / 2
            } else {
                0
            };
            (notification_text, notification_x)
        });

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
                    help_lines.as_ref().map(|v| (v.as_slice(), 2, 2)),
                    status_data,
                    paused_data,
                    notification_data,
                )?;
            } else {
                renderer.render_with_overlay(
                    downsampled.cells(),
                    max_brightness,
                    help_lines.as_ref().map(|v| (v.as_slice(), 2, 2)),
                    status_data,
                    paused_data,
                    notification_data,
                )?;
            }
        }

        timer.end_render();

        let mut should_exit = false;
        let events = input_poller.drain_all_events()?;
        for event in events {
            match event {
                Event::Key(key_event) => {
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
                        ControlAction::CycleOptionsCategory => {
                            runtime_state.cycle_options_category(true);
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
                            current_max_brightness = runtime_state.max_brightness;
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
                            current_max_brightness = runtime_state.max_brightness;
                            renderer.set_invert_palette(runtime_state.invert_palette);
                            renderer.set_reverse_palette(runtime_state.reverse_palette);
                            runtime_state.show_notification("Reset to defaults".to_string());
                        }
                        ControlAction::ShowOptionsOverlay => {
                            if runtime_state.help_mode == HelpMode::Options {
                                runtime_state.toggle_help();
                            } else {
                                runtime_state.help_mode = HelpMode::Options;
                                runtime_state.show_help = true;
                            }
                        }
                        ControlAction::Quit => {
                            should_exit = true;
                            break;
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
