use crossterm::event::{KeyEvent};
use ratatui::{backend::Backend, Frame};

use crate::app::AppCommand;

pub mod home;
pub mod keyboard;

pub trait View<B: Backend> {
    fn render(&mut self, frame: &mut Frame<B>);
    fn on_event(&mut self, event: KeyEvent) -> Option<AppCommand>;
}
