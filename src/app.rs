use color_eyre::Result;
use log::info;

use crate::{
    config::Config,
    tui::{self},
};

pub fn save_log() {
    info!("Saving log file");
    tui_logger::move_events()
}

pub fn run() -> Result<()> {
    let mut config = Config::new();
    let _operation = tui::guide(&mut config)?;

    println!("{:#?}", config);

    save_log();
    Ok(())
}
