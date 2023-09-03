use color_eyre::Result;
use crossterm::event::KeyCode;
use macro_rules_attribute::macro_rules_derive;
use ratatui::layout::{Constraint, Layout};

use crate::tui::{
        data::timezones::get_timezones,
        widgets::{
            menu::{CachedSearchableMenu, MenuArgs},
            Widget,
        },
        Msg, TuiBackend,
    };

use super::{vertical_layout, View, fetch_data_if_needed, WrappedView};

#[derive(Debug, Default)]
#[macro_rules_derive(WrappedView)]
struct Timezone {
    layout: Layout,
    menu: CachedSearchableMenu<String>,
}

impl Timezone {
    fn new() -> Self {
        let layout = vertical_layout([
            Constraint::Length(3),
            Constraint::Min(18),
            Constraint::Length(1),
        ]);

        Self {
            layout,
            ..Default::default()
        }
    }
}

impl View for Timezone {
    fn on_event(
        &mut self,
        event: crossterm::event::KeyEvent,
        _config: &mut crate::config::Config,
    ) -> Option<crate::tui::Msg> {
        match event.code {
            KeyCode::Esc => {
                if self.menu.search_enabled() {
                    self.menu.reset_search();
                    return None;
                }

                Some(Msg::BackToMain)
            }
            KeyCode::Char('q') if !self.menu.search_enabled() => Some(Msg::BackToMain),
            _ => self.menu.on_event(event),
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame<TuiBackend>) -> Result<()> {
        fetch_data_if_needed!({
            let tz = get_timezones()?;
            self.menu.replace_items(tz);
        });

        let chunks = self.layout.split(frame.size());

        self.menu
            .render_default(MenuArgs::builder().frame(frame).area(chunks[1]).build());

        self.menu.render_searchbar_default(frame, chunks[2]);

        Ok(())
    }
}
