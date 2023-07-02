use std::process;

mod tui;
mod cli;
mod config;

fn main() {
    cli::run().unwrap_or_else(|e| {
        eprintln!("{:?}", e);
        process::exit(1)
    })
}