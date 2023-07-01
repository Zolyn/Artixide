use std::{
    collections::HashMap,
    ops::{Deref, Range, RangeBounds},
};

use crossterm::event::Event;
use ratatui::{
    backend::Backend,
    layout::Rect,
    widgets::{self, ListItem, ListState, Widget},
    Frame,
};

pub struct List<'a> {
    raw_items: Vec<&'a str>,
    index_range: Range<usize>,
    state: ListState,
}

impl<'a> List<'a> {
    pub fn new<I: Into<Vec<&'a str>>>(items: I) -> Self {
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
        let mut prev = self.current_index() - 1;

        if !self.index_range.contains(&prev) {
            prev = self.raw_items.len() - 1;
        }

        self.state.select(Some(prev))
    }

    pub fn current_item(&self) -> &str {
        self.raw_items[self.current_index()]
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        self.render_with(frame, area, |l| l);
    }

    pub fn render_with<B: Backend, F: Fn(widgets::List) -> widgets::List>(
        &mut self,
        frame: &mut Frame<B>,
        area: Rect,
        f: F,
    ) {
        let default_instance = widgets::List::new(
            self.raw_items
                .iter()
                .map(|i| ListItem::new(*i))
                .collect::<Vec<_>>(),
        );

        let instance = f(default_instance);

        frame.render_stateful_widget(instance, area, &mut self.state)
    }
}
