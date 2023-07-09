use color_eyre::Result;

use crate::{
    config::Config,
    tui::{self},
};

pub fn run() -> Result<()> {
    let mut config = Config::new();
    let _operation = tui::guide(&mut config)?;

    println!("{:#?}", config);

    Ok(())
}
