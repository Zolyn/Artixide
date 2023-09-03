use std::str::FromStr;

use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use macro_rules_attribute::derive;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
};

use crate::{
    config::Config,
    extensions::{Take, BlockExt},
    lazy,
    tui::{
        views::Route,
        widgets::{
            input::{Input, InputCommand},
            menu::{MenuArgs, SearchableMenu},
             Widget,
        },
        Msg, Operation, TuiBackend,
    }, impl_take,
};

use super::{vertical_layout, View, WrappedView};

lazy! {
    static LAYOUT: Layout = vertical_layout([
        Constraint::Length(3),
        Constraint::Max(18),
        Constraint::Min(1),
    ]);
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

#[derive(Debug, Default, Clone, Copy)]
enum Focus {
    #[default]
    Menu,
    Hostname,
}

impl_take!(Focus);

#[derive(Debug, Default, WrappedView!)]
struct Main {
    menu: SearchableMenu,
    focus: Focus,
    input: Input,
}

impl Main {
    fn new() -> Self {
        Self::default()
    }
}

impl Main {
    fn handle_menu(&mut self, event: KeyEvent, _config: &mut Config) -> Option<Msg> {
        match event.code {
            KeyCode::Enter => {
                let item = ITEMS[self.menu.current_index()?];

                if item == "Hostname" {
                    self.focus = Focus::Hostname;
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

        let focus = self.focus.take();

        if matches!(command, InputCommand::Cancel) {
            self.input.clear();
            return None;
        }

        let input: String = self.input.take();
        self.menu.reset_search();

        match focus {
            Focus::Hostname => config.hostname = input,
            _ => unreachable!(),
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
        match self.focus {
            Focus::Menu => self.handle_menu(event, config),
            Focus::Hostname => self.handle_input(event, config),
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame<TuiBackend>) -> Result<()> {
        let chunks = LAYOUT.split(frame.size());

        let text_area = vertical_layout([Constraint::Length(1); 3]).split(chunks[0])[1];

        let text = Line::from(vec![Span::raw("Select option")]);

        let p = Paragraph::new(text);

        frame.render_widget(p, text_area);

        let menu_area = chunks[1];

        self.menu.render(
            ITEMS,
            MenuArgs::builder().frame(frame).area(chunks[1]).build(),
        );

        let search_area =
            vertical_layout([Constraint::Min(1), Constraint::Length(1)]).split(chunks[2])[1];

        self.menu.render_searchbar_default(frame, search_area);

        if matches!(self.focus, Focus::Menu) {
            return Ok(());
        }

        let offset = self.menu.current_index().unwrap() + 2;

        let title = match self.focus {
            Focus::Hostname => "Hostname",
            _ => unreachable!(),
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

        self.input.render(frame, inner);

        Ok(())
    }
}
