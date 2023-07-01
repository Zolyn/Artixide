use std::{
    collections::HashMap,
    ops::{Deref, Range, RangeBounds, DerefMut},
};
use crate::{views::View, app::AppCommand};

use crossterm::event::{Event, KeyEvent, KeyCode};
use ratatui::{
    backend::Backend,
    layout::Rect,
    widgets::{self, ListItem, ListState, Widget, Block, Borders},
    Frame, style::{Style, Color, Modifier},
};

pub struct Menu {
    raw_items: Vec<String>,
    index_range: Range<usize>,
    state: ListState,
}

impl Menu {
    pub fn new<I: Into<Vec<String>>>(items: I) -> Self {
        let raw_items: Vec<_> = items.into();
        let index_range = 0..raw_items.len();
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            raw_items,
            index_range,
            state,
        }
    }

    fn current_index(&self) -> usize {
        self.state.selected().unwrap()
    }

    pub fn next_item(&mut self) {
        let mut next = self.current_index() + 1;

        if !self.index_range.contains(&next) {
            next = 0;
        }

        self.state.select(Some(next))
    }

    pub fn prev_item(&mut self) {
        let cur = self.current_index();
        let prev = if cur == 0 {
            self.raw_items.len() - 1
        } else {
            cur - 1
        };

        self.state.select(Some(prev))
    }

    pub fn current_item(&self) -> &str {
        &self.raw_items[self.current_index()]
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        let selected_index = self.current_index();

        self.render_with(frame, area, |items| {
            let items = items.into_iter().enumerate().map(|(index, item)| {
                let style = Style::default().fg(Color::White);

                if index == selected_index {
                    item.style(style.bg(Color::LightBlue))
                } else {
                    item.style(style)
                }
            }).collect::<Vec<_>>();

            let list = widgets::List::new(items);

            list.block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::White)))
                .highlight_symbol("> ").highlight_style(Style::default().fg(Color::White))
        });
    }

    pub fn render_with<B: Backend, F: Fn(Vec<ListItem>) -> widgets::List>(
        &mut self,
        frame: &mut Frame<B>,
        area: Rect,
        f: F,
    ) {
        let items = self.raw_items
        .iter()
        .map(|i| ListItem::new(&**i))
        .collect::<Vec<_>>();

        let instance = f(items);

        frame.render_stateful_widget(instance, area, &mut self.state)
    }
}

pub struct MenuView {
    inner: Menu
}

impl MenuView {
    pub fn new<I: Into<Vec<String>>>(items: I) -> Self {
        let inner = Menu::new(items);

        Self { inner }
    }

    pub fn on_event(&mut self, event: KeyEvent) -> Option<AppCommand> {
        match event.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.inner.prev_item();
                None
            },
            KeyCode::Down | KeyCode::Char('j') => {
                self.inner.next_item();
                None
            }
            KeyCode::Char('q') => Some(AppCommand::Shutdown),
            _ => None,
        }
    }
}

impl Deref for MenuView {
    type Target = Menu;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for MenuView{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}