use std::process::Command;

use anyhow::{Result, anyhow, Context};
use crossterm::event::KeyCode;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout},
};

use crate::{tui::{widgets::menu::MenuView, TuiBackend, TuiCommand}, config::Config};

use super::{vertical_layout, View, run_command};

fn get_keyboard_layouts() -> Result<Vec<String>> {
    let output = Command::new("ls")
        .args(["-lR", "/usr/share/kbd/keymaps"])
        .output()?
        .stdout;

    let mut layouts = String::from_utf8_lossy(&output)
        .lines()
        .filter(|line| line.ends_with(".map.gz"))
        .map(|l| l.split(' ').last().unwrap().trim_end_matches(".map.gz").to_string())
        .collect::<Vec<_>>();

    layouts.sort();

    Ok(layouts)
}

pub struct Keyboard {
    menu: MenuView,
    layout: Layout,
}

impl Keyboard {
    pub fn new() -> Self {
        let layout = vertical_layout([Constraint::Length(3), Constraint::Max(24)]);

        let menu = MenuView::new([]);

        Self { layout, menu }
    }
}

impl View<TuiBackend> for Keyboard {
    fn on_event(&mut self, event: crossterm::event::KeyEvent, config: &mut Config) -> Option<crate::tui::TuiCommand> {
        match event.code {
            KeyCode::Enter => {
                config.keyboard_layout = self.menu.current_item().to_string();

                Some(TuiCommand::ChangeRoute("/".to_string()))
            },
            KeyCode::Esc | KeyCode::Char('q') => Some(TuiCommand::ChangeRoute("/".to_string())),
            _ =>
                self.menu.on_event(event)
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame<TuiBackend>) -> Result<()> {
        let chunks = self.layout.split(frame.size());

        let layouts = get_keyboard_layouts().context("Get keyboard layouts")?;

        self.menu.update(|items| *items = layouts);

        self.menu.render(frame, chunks[1]);

        Ok(())
    }
}