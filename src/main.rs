use clap::Parser;
use std::io::{self, Write};

mod cli;
mod render;
mod simulation;
mod terminal;

use cli::{Args, Mode};
use render::charset::Charset;
use render::downsample::downsample;
use simulation::Simulation;
use terminal::input::InputPoller;
use terminal::output::FrameBuffer;
use terminal::screen::TerminalScreen;
use terminal::timing::FrameTimer;

fn main() -> io::Result<()> {
    let args = Args::parse();

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

    sim.update();

    let (term_width, term_height) = get_terminal_size();

    let downsampled = downsample(
        sim.trail_map().current(),
        sim.width(),
        sim.height(),
        term_width,
        term_height,
    );

    let config = args.to_sim_config();

    let buffer = FrameBuffer::from_downsampled(
        downsampled.cells(),
        term_width,
        term_height,
        config.max_brightness,
        palette,
        charset,
    );

    print!("{}", buffer.build_frame_string(args.plain_output));
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

    for frame_idx in 0..args.frame_count {
        for _ in 0..args.frame_skip {
            sim.update();
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
        );

        let frame_content = buffer.build_frame_string(args.plain_output);
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

    let mut renderer = crate::terminal::output::TerminalRenderer::new(0, 0, palette, charset);
    let mut timer = FrameTimer::new(args.fps, args.frame_delay);
    let input_poller = InputPoller::new();

    let (mut term_width, mut term_height) = screen.get_size()?;
    renderer.set_dimensions(term_width as usize, term_height as usize);

    let config = args.to_sim_config();

    loop {
        if screen.check_resize() {
            let (new_width, new_height) = screen.get_size()?;
            if new_width != term_width || new_height != term_height {
                term_width = new_width;
                term_height = new_height;
                renderer.set_dimensions(term_width as usize, term_height as usize);
            }
        }

        sim.update();

        let downsampled = downsample(
            sim.trail_map().current(),
            sim.width(),
            sim.height(),
            term_width as usize,
            term_height as usize,
        );

        if config.max_brightness > 0.0 {
            renderer.render(downsampled.cells(), config.max_brightness)?;
        }

        if let Some(key_event) = input_poller.poll_keypress()? {
            if InputPoller::is_exit_key(&key_event) {
                break;
            }
        }

        if args.verbose {
            eprintln!(
                "FPS: {:.1} | Frame: {} | Agents: {} | Max brightness: {:.2}",
                timer.current_fps(),
                timer.frame_count(),
                sim.agents().len(),
                config.max_brightness
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
