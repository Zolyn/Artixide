use crossterm::event::KeyCode;
use ratatui::{layout::{Layout, Constraint, Direction, Alignment}, backend::Backend, text::{Line, Span}, widgets::Paragraph, style::{Style, Color}};

use crate::{widgets::menu::MenuView, app::AppCommand};

use super::View;

pub struct Home {
    menu: MenuView,
    layout: Layout
}

impl Home {
    pub fn init() -> Self {
        let layout = Layout::default()
        .direction(Direction::Vertical)
        .margin(0)
        .constraints([
            Constraint::Length(3),
            Constraint::Max(24),
            Constraint::Min(1)
        ]);

        let menu = {
            let items = [
                "Keyboard",
                "Partitioning",
                "Bootloader",
                "Timezone"
            ].into_iter().map(|i| i.to_string()).collect::<Vec<_>>();

            MenuView::new(items)
        };

        Self {
            layout,
            menu,
        }

    }
}

impl<B: Backend> View<B> for Home {
    fn on_event(&mut self, event: crossterm::event::KeyEvent) -> Option<crate::app::AppCommand> {
        match event.code {
            KeyCode::Enter => {
                let selected = self.menu.current_item().to_lowercase();

                if selected == "keyboard" {
                    Some(AppCommand::ChangeRoute("/keyboard"))
                } else {
                    None
                }
            },
            _ => self.menu.on_event(event)
        }
    }
    fn render(&mut self, frame: &mut ratatui::Frame<B>) {
        let chunks = self.layout.split(frame.size());

        let text_area = Layout::default().direction(Direction::Vertical).margin(0).constraints([Constraint::Length(1); 3]).split(chunks[0])[1];

        let text = Line::from(vec![Span::styled("Select option", Style::default().fg(Color::White))]);

        let p = Paragraph::new(text);

        frame.render_widget(p, text_area);

        self.menu.render(frame, chunks[1])
    }
}