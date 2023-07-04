use std::process::Command;

use anyhow::{Context, Result};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    config::Config,
    tui::{widgets::menu::MenuView, TuiBackend, TuiCommand},
};

use super::{vertical_layout, View};

fn get_keyboard_layouts() -> Result<Vec<String>> {
    let output = Command::new("ls")
        .args(["-lR", "/usr/share/kbd/keymaps"])
        .output()?
        .stdout;

    let mut layouts = String::from_utf8_lossy(&output)
        .lines()
        .filter(|line| line.ends_with(".map.gz"))
        .map(|l| {
            l.split(' ')
                .last()
                .unwrap()
                .trim_end_matches(".map.gz")
                .to_string()
        })
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
        let layout = vertical_layout([
            Constraint::Length(3),
            Constraint::Min(24),
            Constraint::Length(1),
        ]);

        let menu = MenuView::new([]);

        Self { layout, menu }
    }
}

impl View<TuiBackend> for Keyboard {
    fn on_event(
        &mut self,
        event: crossterm::event::KeyEvent,
        config: &mut Config,
    ) -> Option<crate::tui::TuiCommand> {
        match event.code {
            KeyCode::Enter => {
                config.keyboard_layout = self.menu.current_item()?.to_string();

                Some(TuiCommand::ChangeRoute("/".to_string()))
            }
            KeyCode::Esc | KeyCode::Char('q') => Some(TuiCommand::ChangeRoute("/".to_string())),
            _ => self.menu.on_event(event),
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame<TuiBackend>) -> Result<()> {
        let chunks = self.layout.split(frame.size());

        let _layouts = get_keyboard_layouts().context("Get keyboard layouts")?;

        // self.menu.update(|items| *items = layouts);

        self.menu.render(frame, chunks[1]);

        let search_text = Line::from(vec![Span::raw(self.menu.get_searchbar_text())]);

        let searchbar = Paragraph::new(search_text);

        frame.render_widget(searchbar, chunks[2]);

        Ok(())
    }
}
