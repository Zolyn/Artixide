use crossterm::event::KeyCode;
use ratatui::widgets::{ListState, TableState};

use super::event::WidgetEventHandler;

#[derive(Debug, Default)]
pub struct SelectableWidget<T> {
    last_items_len: usize,
    state: T,
}

impl<T: Selectable> SelectableWidget<T> {
    pub fn update_state(&mut self, len: usize) {
        if len == self.last_items_len {
            return;
        }

        let new_state = if len == 0 {
            None
        } else {
            Some(self.state.selected().unwrap_or(0).min(len - 1))
        };

        self.last_items_len = len;

        // Reset offset and recalculate it when rendering
        *self.state.offset_mut() = 0;
        self.state.select(new_state)
    }

    pub fn select_first_item(&mut self) {
        if self.state.selected().is_some() {
            self.state.select(Some(0))
        }
    }

    pub fn select_last_item(&mut self) {
        if self.state.selected().is_some() {
            self.state.select(Some(self.last_items_len - 1))
        }
    }

    pub fn select_prev_item(&mut self) {
        let Some(cur) = self.state.selected() else { return; };

        let prev = if cur == 0 {
            self.last_items_len - 1
        } else {
            cur - 1
        };

        self.state.select(Some(prev))
    }

    pub fn select_next_item(&mut self) {
        let Some(cur) = self.state.selected() else { return; };

        let mut next = cur + 1;

        if next > self.last_items_len - 1 {
            next = 0;
        }

        self.state.select(Some(next))
    }

    pub fn current_index(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn as_state_mut(&mut self) -> &mut T {
        &mut self.state
    }
}

impl<T: Selectable> WidgetEventHandler for SelectableWidget<T> {
    fn on_event(&mut self, event: crossterm::event::KeyEvent) {
        match event.code {
            KeyCode::Up | KeyCode::Char('k') => self.select_prev_item(),
            KeyCode::Down | KeyCode::Char('j') => self.select_next_item(),
            KeyCode::Home | KeyCode::Char('g') => self.select_first_item(),
            KeyCode::End | KeyCode::Char('G') => self.select_last_item(),
            _ => {}
        };
    }
}

macro_rules! delegate_selection_methods {
    ($self:ident.$field:tt) => {
        delegate::delegate! {
            to $self.$field {
                pub fn select_first_item(&mut $self);
                pub fn select_last_item(&mut $self);
                pub fn select_prev_item(&mut $self);
                pub fn select_next_item(&mut $self);
            }
        }
    };
}

pub(super) use delegate_selection_methods;

pub trait Selectable {
    fn offset(&self) -> usize;
    fn offset_mut(&mut self) -> &mut usize;
    fn selected(&self) -> Option<usize>;
    fn select(&mut self, index: Option<usize>);
}

macro_rules! impl_selectable {
    ($($t:ty),+$(,)*) => {
        $(impl Selectable for $t {
            delegate::delegate! {
                to self {
                    fn offset(&self) -> usize;
                    fn offset_mut(&mut self) -> &mut usize;
                    fn selected(&self) -> Option<usize>;
                    fn select(&mut self, index: Option<usize>);
                }
            }
        })+
    };
}

impl_selectable!(ListState, TableState);
