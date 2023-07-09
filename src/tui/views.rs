use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::config::Config;

use super::TuiCommand;

pub mod keyboard;
pub mod locale;
pub mod main;
pub mod mirror;

pub trait View<B: Backend> {
    fn render(&mut self, frame: &mut Frame<B>) -> Result<()>;
    fn on_event(&mut self, event: KeyEvent, config: &mut Config) -> Option<TuiCommand>;
}

fn make_layout(constraints: Vec<Constraint>, direction: Direction) -> Layout {
    Layout::default()
        .direction(direction)
        .margin(0)
        .constraints(constraints)
}

pub fn vertical_layout<C: Into<Vec<Constraint>>>(constraints: C) -> Layout {
    make_layout(constraints.into(), Direction::Vertical)
}

pub fn horizontal_layout<C: Into<Vec<Constraint>>>(constraints: C) -> Layout {
    make_layout(constraints.into(), Direction::Horizontal)
}
