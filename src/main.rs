use clap::Parser;

mod simulation;
mod render;
mod terminal;
mod cli;

fn main() {
    let args = cli::Args::parse();
    println!("tslime");
    println!("{:?}", args);
}
