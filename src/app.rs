use crate::{config::Config, tui};
use color_eyre::Result;

pub mod logging;
pub mod preparation;

pub fn run() -> Result<()> {
    let mut config = Config::new();
    let _operation = tui::guide(&mut config)?;

    println!("{:#?}", config);

    Ok(())
}
