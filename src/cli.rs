use anyhow::Result;

use crate::{config::Config, tui};

pub fn run() -> Result<()> {
    let mut config = Config::new();
    let operation = tui::guide(&mut config)?;

    Ok(())
}