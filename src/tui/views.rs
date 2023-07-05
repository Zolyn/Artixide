use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::config::Config;

use super::TuiCommand;

pub mod keyboard;
pub mod main;
pub mod mirror;

pub trait View<B: Backend> {
    fn render(&mut self, frame: &mut Frame<B>) -> Result<()>;
    fn on_event(&mut self, event: KeyEvent, config: &mut Config) -> Option<TuiCommand>;
}

pub fn vertical_layout<C: Into<Vec<Constraint>>>(constraints: C) -> Layout {
    let constraints: Vec<Constraint> = constraints.into();
    Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(constraints)
}
