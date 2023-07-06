use std::{
    collections::HashMap,
    env,
    io::{self, Stdout},
    path::PathBuf,
    process::{Command, Output},
};

use anyhow::{anyhow, Context, Result};
use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::skim::SkimMatcherV2;
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::config::Config;

use self::views::{keyboard::Keyboard, main::Main, mirror::Mirror, View};

mod views;
mod widgets;

thread_local! {static FUZZY_MATCHER: SkimMatcherV2 = SkimMatcherV2::default()}

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

fn init(is_tty: bool) -> Result<Terminal<TuiBackend>> {
    enable_raw_mode()?;

    let stdout = io::stdout();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    if is_tty {
        terminal.clear()?;
    } else {
        execute!(terminal.backend_mut(), EnterAlternateScreen)?;
    }

    terminal.hide_cursor()?;

    Ok(terminal)
}

fn close(terminal: &mut Terminal<TuiBackend>, is_tty: bool) -> Result<()> {
    disable_raw_mode()?;

    if is_tty {
        terminal.clear()?;
    } else {
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    }

    terminal.show_cursor()?;

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

pub fn guide(config: &mut Config) -> Result<Operation> {
    let is_tty = is_tty().context("Check tty")?;
    let mut terminal = init(is_tty).context("Init Tui")?;

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
        let view = route_map.get_mut(route.as_str()).unwrap();

        let command = render_view(&mut terminal, view, config).map_err(|err| {
            if let Err(e) = close(&mut terminal, is_tty) {
                err.context(format!("Close Tui: {}", e))
            } else {
                err
            }
        })?;

        match command {
            TuiCommand::ChangeRoute(r) => route = r,
            TuiCommand::Close(operation) => {
                close(&mut terminal, is_tty).context("Close Tui")?;
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

fn run_command(command: &mut Command) -> Result<Output> {
    let output = command.output()?;

    if output.status.success() {
        Ok(output)
    } else {
        let err = if !output.stderr.is_empty() {
            output.stderr
        } else {
            output.stdout
        };

        Err(anyhow!(String::from_utf8_lossy(&err).to_string()))
    }
}
