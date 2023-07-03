use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    config::Config,
    tui::{
        widgets::menu::{to_string_vec, MenuView},
        Operation, TuiBackend, TuiCommand,
    },
};

use super::{vertical_layout, View};

pub struct Main {
    menu: MenuView,
    layout: Layout,
}

impl Main {
    pub fn new() -> Self {
        let layout = vertical_layout([
            Constraint::Length(3),
            Constraint::Max(24),
            Constraint::Min(1),
        ]);

        let menu = {
            let items = to_string_vec([
                "Keyboard layout",
                "Mirror",
                "Locale language",
                "Locale encoding",
                // "Drives",
                "Bootloader",
                // "Swap",
                "Hostname",
                "Timezone",
            ]);

            MenuView::new(items)
        };

        Self { layout, menu }
    }
}

impl View<TuiBackend> for Main {
    fn on_event(
        &mut self,
        event: crossterm::event::KeyEvent,
        _: &mut Config,
    ) -> Option<crate::tui::TuiCommand> {
        match event.code {
            KeyCode::Enter => {
                let selected = format!(
                    "/{}",
                    self.menu.current_item().to_lowercase().replace(' ', "_")
                );

                Some(TuiCommand::ChangeRoute(selected))
            }
            KeyCode::Char('q') => Some(TuiCommand::Close(Operation::Quit)),
            _ => self.menu.on_event(event),
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame<TuiBackend>) -> Result<()> {
        let chunks = self.layout.split(frame.size());

        let text_area = Layout::default()
            .direction(Direction::Vertical)
            .margin(0)
            .constraints([Constraint::Length(1); 3])
            .split(chunks[0])[1];

        let text = Line::from(vec![Span::raw("Select option")]);

        let p = Paragraph::new(text);

        frame.render_widget(p, text_area);

        self.menu.render(frame, chunks[1]);

        Ok(())
    }
}
