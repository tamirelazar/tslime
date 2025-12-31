use clap::Parser;
use std::io::{self, Write};

mod cli;
mod render;
mod simulation;
mod terminal;

use cli::{Args, Mode, ColorMode};
use render::charset::Charset;
use render::downsample::downsample;
use simulation::Simulation;
use simulation::config::{Preset, SimConfig};
use terminal::control::{handle_key_event, num_palettes, ControlAction, RuntimeState};
use terminal::input::InputPoller;
use terminal::output::FrameBuffer;
use terminal::screen::TerminalScreen;
use terminal::signal::is_shutdown_requested;
use terminal::timing::FrameTimer;

const REFERENCE_TIME_STEP: f32 = 1.0 / 30.0;

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
    );

    let mode = args.mode();

    if mode == Mode::Print {
        print_mode(&mut sim, &args, palette, charset)?;
    } else if mode == Mode::CaptureFrames {
        capture_frames_mode(&mut sim, &args, palette, charset)?;
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

    let downsampled = downsample(
        sim.trail_map().current(),
        sim.width(),
        sim.height(),
        term_width,
        term_height,
    );

    let config = args.to_sim_config();
    let color_mode = args.color_mode().unwrap_or(ColorMode::Bits256);

    let buffer = FrameBuffer::from_downsampled(
        downsampled.cells(),
        term_width,
        term_height,
        config.max_brightness,
        palette,
        charset,
        args.reverse_palette,
        args.invert_palette,
        color_mode,
    );

    print!("{}", buffer.build_frame_string(args.plain_output, color_mode));
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

    for frame_idx in 0..args.frame_count {
        for _ in 0..args.frame_skip {
            sim.update(1.0);
        }

        let downsampled = downsample(
            sim.trail_map().current(),
            sim.width(),
            sim.height(),
            term_width,
            term_height,
        );

        let buffer = FrameBuffer::from_downsampled(
            downsampled.cells(),
            term_width,
            term_height,
            config.max_brightness,
            palette.clone(),
            charset,
            args.reverse_palette,
            args.invert_palette,
            color_mode,
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
    ];
    let initial_palette_index = palette_list.iter().position(|p| *p == initial_palette).unwrap_or(4);

    let mode = args.mode();
    let show_help_by_default = !matches!(mode, cli::Mode::Screensaver);

    let mut runtime_state = RuntimeState::new(
        seed,
        args.init,
        initial_preset,
        initial_palette_index,
        show_help_by_default,
    );

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
            sim.update(dt / REFERENCE_TIME_STEP);
        }

        let downsampled = downsample(
            sim.trail_map().current(),
            sim.width(),
            sim.height(),
            term_width as usize,
            term_height as usize,
        );

        let current_config = args.to_sim_config();
        let current_palette = runtime_state.current_palette(&palette_list);

        static HELP_LINES: [&str; 9] = [
            "┌─ tslime controls ───────────────────────┐",
            "│ p: Pause/Resume                         │",
            "│ r: Restart                              │",
            "│ 1-4: Presets  (Network,Exploratory,etc) │",
            "│ +/-: Time scale (0.5x - 4.0x)           │",
            "│ c: Cycle palette (Shift+C reverse)      │",
            "│ h: Toggle this help                     │",
            "│ q: Quit                                 │",
            "└─────────────────────────────────────────┘",
        ];

        let help_data = if runtime_state.show_help {
            Some((HELP_LINES.as_slice(), 2, 2))
        } else {
            None
        };

        let status_line = render::overlay::OverlayRenderer::build_status_line(
            runtime_state.is_paused,
            runtime_state.current_preset,
            runtime_state.time_scale,
            current_palette,
            term_width as usize,
        );
        let status_x = render::overlay::OverlayRenderer::status_line_x(&status_line, term_width as usize);
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

        if current_config.max_brightness > 0.0 {
            renderer.render_with_overlay(
                downsampled.cells(),
                current_config.max_brightness,
                help_data,
                status_data,
                paused_data,
            )?;
        }

        if let Some(key_event) = input_poller.poll_keypress()? {
            if InputPoller::is_exit_key(&key_event) {
                break;
            }

            let action = handle_key_event(&key_event);
            match action {
                ControlAction::TogglePause => {
                    runtime_state.toggle_pause();
                }
                ControlAction::Restart => {
                    sim.reset(runtime_state.original_seed, runtime_state.original_init_mode);
                }
                ControlAction::SetPreset(preset) => {
                    runtime_state.set_preset(preset);
                    let new_config = SimConfig::from(preset);
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
                ControlAction::ToggleHelp => {
                    runtime_state.toggle_help();
                }
                ControlAction::Quit => {
                    break;
                }
                ControlAction::None => {}
            }
        }

        if args.verbose {
            eprintln!(
                "FPS: {:.1} | Frame: {} | Agents: {} | Max brightness: {:.2}",
                timer.current_fps(),
                timer.frame_count(),
                sim.agents().len(),
                current_config.max_brightness
            );
        }

        timer.tick();
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
    use super::*;

    const HELP_LINES_TOP: &str = "┌─ tslime controls ───────────────────────┐";
    const HELP_LINES_BOTTOM: &str = "└─────────────────────────────────────────┘";
    const HELP_LINES_CONTENT: [&str; 7] = [
        "│ p: Pause/Resume                         │",
        "│ r: Restart                              │",
        "│ 1-4: Presets  (Network,Exploratory,etc) │",
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
