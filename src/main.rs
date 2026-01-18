mod cli;
mod config_manager;
mod export;
mod render;
mod simulation;
mod terminal;

use std::io;

fn main() -> io::Result<()> {
    tslime::app::run()
}
