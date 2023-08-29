use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout};

use crate::{
    fetch_data_if_needed,
    tui::{
        data::timezones::get_timezones,
        widgets::{
            menu::{CachedSearchableMenu, MenuArgs},
            Widget,
        },
        Msg, TuiBackend,
    },
    wrap_view,
};

use super::{vertical_layout, View};

wrap_view!(TimezoneView, Timezone);

#[derive(Debug, Default)]
struct TimezoneView {
    layout: Layout,
    menu: CachedSearchableMenu<String>,
}

impl TimezoneView {
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

impl View for TimezoneView {
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
