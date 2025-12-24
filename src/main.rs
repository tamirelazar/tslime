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

    let mut sim = Simulation::new(args.resolution.width, args.resolution.height, config, seed);

    let mode = args.mode();

    if mode == Mode::Print {
        print_mode(&mut sim, &args, palette, charset)?;
    } else {
        run_simulation(&mut sim, &args, mode, palette, charset)?;
    }

    Ok(())
}

fn print_mode(
    sim: &mut Simulation,
    _args: &Args,
    palette: cli::Palette,
    charset: Charset,
) -> io::Result<()> {
    sim.update();

    let (term_width, term_height) = get_terminal_size();

    let downsampled = downsample(
        sim.trail_map().current(),
        sim.width(),
        sim.height(),
        term_width,
        term_height,
    );

    let max_brightness = downsampled
        .cells()
        .iter()
        .map(|c| c.top.max(c.bottom))
        .fold(0.0f32, |acc, v| acc.max(v));

    let buffer = FrameBuffer::from_downsampled(
        downsampled.cells(),
        term_width,
        term_height,
        max_brightness,
        palette,
        charset,
    );

    print!("{}", buffer.build_frame_string());
    io::stdout().flush()?;

    Ok(())
}

#[allow(unused_assignments)]
fn run_simulation(
    sim: &mut Simulation,
    args: &Args,
    mode: Mode,
    palette: cli::Palette,
    charset: Charset,
) -> io::Result<()> {
    let mut screen = TerminalScreen::new();
    screen.setup()?;

    let mut renderer = crate::terminal::output::TerminalRenderer::new(0, 0, palette, charset);
    let mut timer = FrameTimer::new(args.fps, args.frame_delay);
    let input_poller = if mode == Mode::Screensaver {
        Some(InputPoller::new())
    } else {
        None
    };

    let (mut term_width, mut term_height) = screen.get_size()?;
    renderer.set_dimensions(term_width as usize, term_height as usize);

    let mut max_brightness = 0.0;

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

        max_brightness = downsampled
            .cells()
            .iter()
            .map(|c| c.top.max(c.bottom))
            .fold(0.0f32, |acc, v| acc.max(v).max(acc));

        if max_brightness > 0.0 {
            renderer.render(downsampled.cells(), max_brightness)?;
        }

        if let Some(poller) = &input_poller {
            if poller.poll_keypress()? {
                break;
            }
        }

        if args.verbose {
            eprintln!(
                "FPS: {:.1} | Frame: {} | Agents: {} | Max brightness: {:.2}",
                timer.current_fps(),
                timer.frame_count(),
                sim.agents().len(),
                max_brightness
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
