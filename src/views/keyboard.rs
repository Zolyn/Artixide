use std::process::Command;

use anyhow::Result;
use ratatui::{layout::{Layout, Direction, Constraint}, backend::Backend};

use crate::widgets::menu::MenuView;

use super::View;

fn get_keyboard_layouts() -> Result<Vec<String>> {
    let output = Command::new("ls").args(["-lR", "/usr/share/kbd/keymaps"]).output()?.stdout;
    let layouts = String::from_utf8_lossy(&output).lines().filter(|line| line.ends_with(".map.gz")).map(|l| l.to_owned()).collect::<Vec<_>>();

    Ok(layouts)
}

pub struct Keyboard {
    menu: MenuView,
    layout: Layout
}

impl Keyboard {
    pub fn init() -> Result<Self> {
        let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(3),
            Constraint::Max(24),
            Constraint::Min(1)
        ]);

        let menu = MenuView::new(get_keyboard_layouts()?);

        Ok(Self {
            layout,
            menu,
        })
    }
}

impl<B: Backend> View<B> for Keyboard {
    fn on_event(&mut self, event: crossterm::event::KeyEvent) -> Option<crate::app::AppCommand> {
        self.menu.on_event(event)
    }
    fn render(&mut self, frame: &mut ratatui::Frame<B>) {
        let chunks = self.layout.split(frame.size());

        self.menu.render(frame, chunks[1]);
    }
}
