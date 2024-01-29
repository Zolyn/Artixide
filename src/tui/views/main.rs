use std::str::FromStr;

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use macro_rules_attribute::derive;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph}, style::{Style, Color},
};

use crate::{
    config::Config,
    extensions::BlockExt,
    lazy,
    tui::{
        views::Route,
        widgets::{
            input::{Input, InputCommand, InputArgs},
            menu::{MenuArgs, SearchableMenu},
             Widget,
        },
        Msg, Operation, TuiBackend,
    }
};

use super::{vertical_layout, View, WrappedView};

lazy! {
    static LAYOUT: Layout = vertical_layout([
        Constraint::Length(3),
        Constraint::Max(18),
        Constraint::Min(1),
    ]);

    static TEXT_LAYOUT: Layout = vertical_layout([Constraint::Length(1); 3]);
    static SEARCH_LAYOUT: Layout = vertical_layout([Constraint::Min(1), Constraint::Length(1)]);
}

const ITEMS: &[&str] = &[
    "Keyboard layout",
    "Mirror",
    "Locale",
    "Partition",
    // "Bootloader",
    // "Swap",
    "Hostname",
    "Timezone",
    "Init",
];

#[derive(Debug, Clone, Copy)]
enum Popup {
    Hostname,
}

#[derive(Debug, Default, WrappedView!)]
struct Main {
    menu: SearchableMenu,
    popup: Option<Popup>,
    // Shared
    input: Input,
}

impl Main {
    fn handle_menu(&mut self, event: KeyEvent, _config: &mut Config) -> Option<Msg> {
        match event.code {
            KeyCode::Enter => {
                let item = ITEMS[self.menu.current_index()?];

                if item == "Hostname" {
                    self.popup = Some(Popup::Hostname);
                    return None;
                }

                let route = Route::from_str(item).unwrap();

                self.menu.reset_search();

                Some(Msg::ChangeRoute(route))
            }
            KeyCode::Esc if self.menu.search_enabled() => {
                self.menu.reset_search();
                None
            }
            KeyCode::Char('q') => Some(Msg::Close(Operation::Quit)),
            _ => self.menu.on_event(event),
        }
    }

    fn handle_input(&mut self, event: KeyEvent, config: &mut Config) -> Option<Msg> {
        let command = self.input.on_event(event)?;

        let popup = self.popup.take().unwrap();

        if matches!(command, InputCommand::Cancel) {
            self.input.clear();
            return None;
        }

        let input: String = self.input.take();
        self.menu.reset_search();

        match popup {
            Popup::Hostname => config.hostname = input,
        }

        None
    }
}

impl View for Main {
    fn on_event(
        &mut self,
        event: crossterm::event::KeyEvent,
        config: &mut Config,
    ) -> Option<crate::tui::Msg> {
        let Some(popup) = self.popup else { return self.handle_menu(event, config) };

        match popup {
            Popup::Hostname => self.handle_input(event, config),
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame<TuiBackend>) -> Result<()> {
        let chunks = LAYOUT.split(frame.size());
        let text_area = TEXT_LAYOUT.split(chunks[0])[1];

        let text = Line::from(vec![Span::raw("Select option")]);

        let p = Paragraph::new(text);

        frame.render_widget(p, text_area);

        let menu_area = chunks[1];

        self.menu.render(
            ITEMS,
            MenuArgs::builder().frame(frame).area(chunks[1]).build(),
        );

        let search_area = SEARCH_LAYOUT.split(chunks[2])[1];

        self.menu.render_searchbar_default(frame, search_area);

        if self.popup.is_none() {
            return Ok(());
        }

        let offset = self.menu.inner_index().unwrap() + 2;

        let title = match self.popup.unwrap() {
            Popup::Hostname => "Hostname",
        };

        let block = Block::with_borders().title(title);

        let area = Rect {
            y: menu_area.y + offset as u16,
            height: 3,
            ..menu_area
        };

        let inner = block.inner(area);

        frame.render_widget(Clear, area);
        frame.render_widget(block, area);

        self.input.render(InputArgs::builder().frame(frame).area(inner).build());

        Ok(())
    }
}
