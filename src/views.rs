use crossterm::event::{Event, KeyEvent};
use ratatui::{backend::Backend, Frame};

use crate::app::AppCommand;

pub mod home;

pub trait View<B: Backend> {
    fn render(&mut self, frame: &mut Frame<B>);
    fn on_event(&mut self, event: KeyEvent) -> Option<AppCommand>;
}
