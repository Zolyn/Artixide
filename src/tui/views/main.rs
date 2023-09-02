use std::str::FromStr;

use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
};

use crate::{
    config::Config,
    string::StringExt,
    tui::{
        route::Route,
        widgets::{
            input::{Input, InputCommand},
            menu::{MenuArgs, SearchableMenu},
            BlockExt, Widget,
        },
        Msg, Operation, TuiBackend,
    },
    wrap_view,
};

use super::{vertical_layout, View};

wrap_view!(MainView, Main);

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
enum InputType {
    Hostname,
}

#[derive(Debug)]
struct MainView {
    menu: SearchableMenu,
    layout: Layout,
    input_type: Option<InputType>,
    input: Input,
}

impl MainView {
    fn new() -> Self {
        let layout = vertical_layout([
            Constraint::Length(3),
            Constraint::Max(18),
            Constraint::Min(1),
        ]);

        let menu = SearchableMenu::default();

        Self {
            layout,
            menu,
            input_type: None,
            input: Input::default(),
        }
    }
}

// TODO:
impl View for MainView {
    fn on_event(
        &mut self,
        event: crossterm::event::KeyEvent,
        config: &mut Config,
    ) -> Option<crate::tui::Msg> {
        if let Some(input_type) = self.input_type {
            let command = self.input.on_event(event)?;

            self.input_type.take();

            if matches!(command, InputCommand::Cancel) {
                self.input.clear();
                return None;
            }

            let input = self.input.take();
            self.menu.reset_search();

            match input_type {
                InputType::Hostname => config.hostname = input,
            }

            return None;
        }

        match event.code {
            KeyCode::Enter => {
                let item = ITEMS[self.menu.current_index()?];

                if item == "Hostname" {
                    self.input_type = Some(InputType::Hostname);
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

    fn render(&mut self, frame: &mut ratatui::Frame<TuiBackend>) -> Result<()> {
        let chunks = self.layout.split(frame.size());

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

        if self.input_type.is_none() {
            return Ok(());
        }

        let offset = self.menu.current_index().unwrap() + 2;

        let title = match self.input_type.unwrap() {
            InputType::Hostname => "Hostname",
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
