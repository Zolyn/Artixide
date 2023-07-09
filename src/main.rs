use std::{
    env, fs, panic,
    path::Path,
    sync::atomic::Ordering,
    time::{SystemTime, UNIX_EPOCH},
};

use color_eyre::{eyre::eyre, Result};
use log::debug;
use sudo::RunningAs;
use tui::TUI_RUNNING;

mod app;
mod config;
mod tui;

fn main() -> Result<()> {
    color_eyre::install()?;

    tui_logger::init_logger(log::LevelFilter::Debug).map_err(|_| eyre!("Failed to init logger"))?;

    tui_logger::set_default_level(log::LevelFilter::Debug);

    let sys_time = SystemTime::now();

    let log_dir = match sudo::check() {
        RunningAs::Root | RunningAs::Suid => Path::new("/var/log/artixide").to_path_buf(),
        RunningAs::User => {
            let mut cwd = env::current_dir()?;

            cwd.push(".artixide_logs");
            cwd
        }
    };

    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)?
    }

    let log_file_name = format!(
        "{}/{}.log",
        log_dir.as_path().to_str().unwrap(),
        sys_time.duration_since(UNIX_EPOCH)?.as_millis()
    );

    tui_logger::set_log_file(&log_file_name)?;

    let hook = panic::take_hook();

    panic::set_hook(Box::new(move |info| {
        debug!("Handling panic");

        if TUI_RUNNING.load(Ordering::SeqCst) {
            tui::destroy().unwrap();
        }

        tui_logger::move_events();
        hook(info)
    }));

    app::run()
}
