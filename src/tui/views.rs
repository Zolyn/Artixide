use std::process::Command;

use anyhow::{Result, anyhow};
use crossterm::event::KeyEvent;
use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::config::Config;

use super::TuiCommand;

pub mod main;
pub mod keyboard;

pub trait View<B: Backend> {
    fn render(&mut self, frame: &mut Frame<B>) -> Result<()>;
    fn on_event(&mut self, event: KeyEvent, config: &mut Config) -> Option<TuiCommand>;
}

pub fn vertical_layout<C: Into<Vec<Constraint>>>(constraints: C) -> Layout {
    let mut constraints: Vec<Constraint> = constraints.into();
    constraints.push(Constraint::Min(1));
    Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints(constraints)
}

pub fn run_command(mut command: Command) -> Result<String> {
    let output = command.output()?;

    if !output.status.success() {
        Err(anyhow!(String::from_utf8_lossy(&output.stderr).to_string()))
    } else {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}