use std::rc::Rc;

use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    backend::Backend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};

use crate::{app::AppCommand, widgets::list::List};

use super::View;

pub struct Home<'a> {
    list: List<'a>,
    layout: Layout,
}

impl<'a> Home<'a> {
    pub fn init() -> Self {
        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .margin(0)
            .constraints([
                Constraint::Percentage(12),
                Constraint::Percentage(70),
                Constraint::Percentage(18),
            ]);

        let list = List::new(["Keyboard settings", "Mirror", "Partitioning", "Bootloader"]);

        Self { list, layout }
    }
}

impl<'a, B: Backend> View<B> for Home<'a> {
    fn on_event(&mut self, event: KeyEvent) -> Option<AppCommand> {
        match event.code {
            KeyCode::Char(c) if c == 'q' => Some(AppCommand::Shutdown),
            _ => None,
        }
    }
    fn render(&mut self, frame: &mut Frame<B>) {
        let chunks = self.layout.split(frame.size());

        self.list.render_with(frame, chunks[1], |list| {
            list.block(Block::default().borders(Borders::ALL))
                .repeat_highlight_symbol(true)
                .highlight_symbol("> ")
        });
    }
}
