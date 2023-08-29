use std::{
    collections::HashMap,
    env,
    io::{self, Stdout},
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use crossterm::{
    cursor,
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use log::info;
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::{config::Config, lazy};

use self::views::{
    keyboard::Keyboard, locale::Locale, main::Main, mirror::Mirror, partition::Partition,
    timezone::Timezone, View,
};

mod data;
mod views;
mod widgets;

lazy! {
    static IS_TTY: bool = is_tty().unwrap();
}

pub static TUI_RUNNING: AtomicBool = AtomicBool::new(true);

#[derive(Debug)]
pub enum Operation {
    SaveAs(PathBuf),
    Install,
    Quit,
}

#[derive(Debug)]
pub enum Msg {
    Close(Operation),
    ChangeRoute(String),
    BackToMain,
}

type TuiBackend = CrosstermBackend<Stdout>;

fn init() -> Result<Terminal<TuiBackend>> {
    enable_raw_mode()?;

    let stdout = io::stdout();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    if *IS_TTY {
        terminal.clear()?;
    } else {
        execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    }

    terminal.hide_cursor()?;

    Ok(terminal)
}

pub fn destroy() -> Result<()> {
    info!("Destructing terminal");
    disable_raw_mode()?;

    let mut stdout = io::stdout();

    if !*IS_TTY {
        execute!(stdout, LeaveAlternateScreen)?;
    }

    execute!(stdout, cursor::Show)?;

    Ok(())
}

/// Check if the current environment is in a TTY or terminal emulator
///
/// The program is supposed to be run in TTY
///
/// But usually we run it in a terminal emulator for testing
fn is_tty() -> Result<bool> {
    let term = env::var("TERM")?;

    Ok(term == "linux")
}

macro_rules! make_route_map {
    ($($route:literal $view:ty),+ $(,)?) => {
        <HashMap<&'static str, Box<dyn View>>>::from([
            $(
                ($route, <$view>::init()),
            )+
        ])
    };
}

pub fn guide(config: &mut Config) -> Result<Operation> {
    let mut terminal = init().wrap_err("Init Tui")?;

    let mut route_map = make_route_map! {
        "/" Main,
        "/keyboard_layout" Keyboard,
        "/mirror" Mirror,
        "/locale" Locale,
        "/timezone" Timezone,
        "/partition" Partition
    };

    let mut route = "/".to_string();

    loop {
        let view = route_map.get_mut(route.as_str()).unwrap();

        let command = render_view(&mut terminal, view, config)
            .wrap_err_with(|| eyre!("Failed to render route: {}", route))?;

        match command {
            Msg::ChangeRoute(r) => route = r,
            Msg::BackToMain => route = "/".to_string(),
            Msg::Close(operation) => {
                TUI_RUNNING.store(false, Ordering::Relaxed);

                destroy().wrap_err("Close Tui")?;
                break Ok(operation);
            }
        }
    }
}

fn render_view(
    terminal: &mut Terminal<TuiBackend>,
    view: &mut Box<dyn View>,
    config: &mut Config,
) -> Result<Msg> {
    let mut err = None;

    loop {
        terminal.draw(|f| {
            if let Err(e) = view.render(f) {
                err = Some(e)
            }
        })?;

        if let Some(err) = err {
            return Err(err);
        }

        if let Event::Key(key_event) = event::read().wrap_err("Read events")? {
            let command = view.on_event(key_event, config);

            if let Some(command) = command {
                break Ok(command);
            }
        }
    }
}
