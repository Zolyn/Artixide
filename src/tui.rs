use std::{
    collections::HashMap,
    io::{self, Stdout},
    path::PathBuf,
};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::config::Config;

use self::views::{keyboard::Keyboard, main::Main, mirror::Mirror, View};

mod views;
mod widgets;

pub enum Operation {
    SaveAs(PathBuf),
    Install,
    Quit,
}

pub enum TuiCommand {
    Close(Operation),
    ChangeRoute(String),
}

type TuiBackend = CrosstermBackend<Stdout>;

fn init() -> Result<Terminal<TuiBackend>> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();

    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    Ok(terminal)
}

fn close(terminal: &mut Terminal<TuiBackend>) -> Result<()> {
    disable_raw_mode()?;

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    terminal.show_cursor()?;

    Ok(())
}

pub fn guide(config: &mut Config) -> Result<Operation> {
    let mut terminal = init().context("Init Tui")?;

    let _close_tui = false;

    let routes: Vec<(&'static str, Box<dyn View<TuiBackend>>)> = vec![
        ("/", Box::new(Main::new())),
        ("/keyboard_layout", Box::new(Keyboard::new())),
        ("/mirror", Box::new(Mirror::new())),
    ];

    let mut route_map: HashMap<&'static str, Box<dyn View<TuiBackend>>> =
        routes.into_iter().collect();
    let mut route = "/".to_string();

    loop {
        let view = route_map.get_mut(&*route).unwrap();

        let command = render_view(&mut terminal, view, config).map_err(|err| {
            if let Err(e) = close(&mut terminal) {
                err.context(format!("Close Tui: {}", e))
            } else {
                err
            }
        })?;

        match command {
            TuiCommand::ChangeRoute(r) => route = r,
            TuiCommand::Close(operation) => {
                close(&mut terminal).context("Close Tui")?;
                break Ok(operation);
            }
        }
    }
}

fn render_view(
    terminal: &mut Terminal<TuiBackend>,
    view: &mut Box<dyn View<TuiBackend>>,
    config: &mut Config,
) -> Result<TuiCommand> {
    let mut render_error: Option<anyhow::Error> = None;

    loop {
        terminal.draw(|f| {
            if let Err(err) = view.render(f) {
                render_error = Some(err)
            }
        })?;

        if let Some(e) = render_error {
            break Err(e);
        }

        if let Event::Key(key_event) = event::read()? {
            let command = view.on_event(key_event, config);

            if let Some(command) = command {
                break Ok(command);
            }
        }
    }
}
