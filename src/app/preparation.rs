use std::{
    env, fs,
    io::{stdout, Write},
    panic,
    path::Path,
    sync::atomic::Ordering,
    time::{SystemTime, UNIX_EPOCH},
};

use color_eyre::{eyre::eyre, Result};
use crossterm::style::Stylize;
use log::debug;
use sudo::RunningAs;

use crate::tui::{self, TUI_RUNNING};

pub fn init_logger() -> Result<()> {
    color_eyre::install()?;

    tui_logger::init_logger(log::LevelFilter::Debug).map_err(|_| eyre!("Failed to init logger"))?;

    tui_logger::set_default_level(log::LevelFilter::Debug);

    Ok(())
}

pub fn escalate_if_needed() -> Result<()> {
    if let RunningAs::User = sudo::check() {
        println!(
            "{} Non-root user detected. Try escalating...",
            "NOTICE:".green()
        );
        stdout().flush()?;
    }

    sudo::escalate_if_needed().map_err(|e| eyre!("Failed to escalate: {}", e))?;

    Ok(())
}

pub fn create_log_file() -> Result<()> {
    let log_dir = if cfg!(debug_assertions) {
        let mut cwd = env::current_dir()?;

        cwd.push(".artixide_logs");
        cwd
    } else {
        Path::new("/var/log/artixide").to_path_buf()
    };

    let sys_time = SystemTime::now();

    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)?
    }

    let log_file_name = format!(
        "{}/{}.log",
        log_dir.as_path().to_str().unwrap(),
        sys_time.duration_since(UNIX_EPOCH)?.as_millis()
    );

    tui_logger::set_log_file(&log_file_name)?;

    Ok(())
}

pub fn set_panic_hook() {
    let hook = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        debug!("Start handling panic");

        if TUI_RUNNING.load(Ordering::Relaxed) {
            tui::destroy().unwrap();
        }

        debug!("Running color_eyre panic hook");
        hook(info)
    }));
}
