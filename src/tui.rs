use std::{
    collections::HashMap,
    env,
    io::{self, Stdout},
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
};

use color_eyre::{
    eyre::{eyre, Context},
    Help, Result, SectionExt,
};
use crossterm::{
    cursor,
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use fuzzy_matcher::skim::SkimMatcherV2;
use log::info;
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::config::Config;

use self::views::{keyboard::Keyboard, locale::Locale, main::Main, mirror::Mirror, View};

mod views;
mod widgets;

thread_local! {
    static FUZZY_MATCHER: SkimMatcherV2 = SkimMatcherV2::default();
}

static IS_TTY: AtomicBool = AtomicBool::new(false);
pub static TUI_RUNNING: AtomicBool = AtomicBool::new(true);

pub enum Operation {
    SaveAs(PathBuf),
    Install,
    Quit,
}

pub enum TuiCommand {
    Close(Operation),
    ChangeRoute(String),
    BackToMain,
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

pub fn destroy() -> Result<()> {
    info!("Destructing terminal");
    disable_raw_mode()?;

    let mut stdout = io::stdout();

    let is_tty = IS_TTY.load(Ordering::SeqCst);

    if !is_tty {
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

pub fn guide(config: &mut Config) -> Result<Operation> {
    let is_tty = is_tty().wrap_err("Check tty")?;

    IS_TTY
        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(is_tty))
        .unwrap();

    let mut terminal = init(is_tty).wrap_err("Init Tui")?;

    let routes: Vec<(&'static str, Box<dyn View<TuiBackend>>)> = vec![
        ("/", Box::new(Main::new())),
        ("/keyboard_layout", Box::new(Keyboard::new())),
        ("/mirror", Box::new(Mirror::new())),
        ("/locale", Box::new(Locale::new())),
    ];

    let mut route_map: HashMap<&'static str, Box<dyn View<TuiBackend>>> =
        routes.into_iter().collect();
    let mut route = "/".to_string();

    loop {
        let view = route_map.get_mut(route.as_str()).unwrap();

        let command = render_view(&mut terminal, view, config)
            .wrap_err_with(|| eyre!("Failed to render view: {}", route))
            .unwrap();

        match command {
            TuiCommand::ChangeRoute(r) => route = r,
            TuiCommand::BackToMain => route = "/".to_string(),
            TuiCommand::Close(operation) => {
                destroy().wrap_err("Close Tui")?;

                TUI_RUNNING
                    .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |_| Some(false))
                    .unwrap();
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
    let mut render_error: Option<color_eyre::Report> = None;

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

fn run_command(command: &mut Command) -> Result<String> {
    let output = command.output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    if output.status.success() {
        return Ok(stdout.into());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);

    Err(eyre!("Command execution failure")
        .with_section(|| stdout.trim().to_string().header("Stdout"))
        .with_section(|| stderr.trim().to_string().header("Stderr")))
}
