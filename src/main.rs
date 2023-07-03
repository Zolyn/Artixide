use std::process;

mod cli;
mod config;
mod tui;

fn main() {
    // let p = env::current_exe().unwrap();
    cli::run().unwrap_or_else(|e| {
        eprintln!("{:?}", e);
        process::exit(1)
    })
    // println!("{}", p.display());
}
