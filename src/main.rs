//#![allow(unused)]
use color_eyre::Result;
use scopeguard::defer;

use crate::app::{
    logging::save_log,
    preparation::{create_log_file, escalate_if_needed, init_logger, set_panic_hook},
    run,
};

mod app;
mod config;
mod extensions;
mod macros;
mod tui;

fn main() -> Result<()> {
    init_logger()?;
    defer!(save_log());

    escalate_if_needed()?;
    create_log_file()?;
    set_panic_hook();
    run()
}
