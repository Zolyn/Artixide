use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{backend::Backend, layout::Rect, text::Line, widgets::Paragraph, Frame};
use unicode_width::UnicodeWidthStr;

use crate::extensions::Take;

pub enum InputCommand {
    Cancel,
    Submit,
}

#[derive(Debug, Default)]
pub struct Input {
    input: String,
}

impl Input {
    pub fn render<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let input = Paragraph::new(Line::from(self.input.as_str()));

        frame.render_widget(input, area);

        frame.set_cursor(area.x + self.input.width() as u16, area.y)
    }

    pub fn on_event(&mut self, event: KeyEvent) -> Option<InputCommand> {
        match event.code {
            KeyCode::Enter => {
                if self.input.is_empty() {
                    Some(InputCommand::Cancel)
                } else {
                    Some(InputCommand::Submit)
                }
            }
            KeyCode::Backspace => {
                if self.input.is_empty() {
                    Some(InputCommand::Cancel)
                } else {
                    self.input.pop();
                    None
                }
            }
            KeyCode::Char(c) => {
                self.input.push(c);
                None
            }
            KeyCode::Esc => Some(InputCommand::Cancel),
            _ => None,
        }
    }

    pub fn take(&mut self) -> String {
        self.input.take()
    }
}

impl Input {
    delegate::delegate! {
        to self.input {
            pub fn as_str(&self) -> &str;
            pub fn clear(&mut self);
            pub fn push(&mut self, c: char);
        }
    }
}
