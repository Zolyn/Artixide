use std::process::Command;

use anyhow::{Context, Result};
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout};

use crate::{
    config::Config,
    tui::{run_command, widgets::menu::MenuView, TuiBackend, TuiCommand},
};

use super::{vertical_layout, View};

fn get_keyboard_layouts() -> Result<Vec<String>> {
    let mut command = Command::new("ls");
    command.args(["-lR", "/usr/share/kbd/keymaps"]);

    let stdout = run_command(&mut command)?.stdout;
    let output = String::from_utf8_lossy(&stdout);

    let mut layouts = output
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
    need_update: bool,
}

impl Keyboard {
    pub fn new() -> Self {
        let layout = vertical_layout([
            Constraint::Length(3),
            Constraint::Min(24),
            Constraint::Length(1),
        ]);

        let menu = MenuView::new([]);

        Self {
            layout,
            menu,
            need_update: true,
        }
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

                Some(TuiCommand::BackToMain)
            }
            KeyCode::Esc => {
                if self.menu.search_mode() {
                    self.menu.disable_search()
                }

                Some(TuiCommand::BackToMain)
            }
            KeyCode::Char('q') if !self.menu.search_mode() => Some(TuiCommand::BackToMain),
            _ => self.menu.on_event(event),
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame<TuiBackend>) -> Result<()> {
        let chunks = self.layout.split(frame.size());

        if self.need_update {
            let layouts = get_keyboard_layouts().context("Get keyboard layouts")?;

            self.menu.replace_items_with(layouts);

            self.need_update = false;
        }

        self.menu.render(frame, chunks[1]);

        self.menu.render_searchbar(frame, chunks[2]);

        Ok(())
    }
}
