use std::process;

mod cli;
mod config;
mod tui;

fn main() {
    cli::run().unwrap_or_else(|e| {
        eprintln!("{:?}", e);
        process::exit(1)
    })
}
