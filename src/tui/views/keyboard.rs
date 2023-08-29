use color_eyre::{eyre::Context, Result};
use crossterm::event::KeyCode;

use ratatui::layout::{Constraint, Layout};

use crate::{
    config::Config,
    fetch_data_if_needed,
    tui::{
        data::keyboard::get_keyboard_layouts,
        widgets::{
            menu::{CachedSearchableMenu, MenuArgs},
            Widget,
        },
        Msg, TuiBackend,
    },
    wrap_view,
};

use super::{vertical_layout, View};

wrap_view!(KeyboardInner, Keyboard);

#[derive(Debug, Default)]
struct KeyboardInner {
    menu: CachedSearchableMenu<String>,
    layout: Layout,
}

impl KeyboardInner {
    fn new() -> Self {
        let layout = vertical_layout([
            Constraint::Length(3),
            Constraint::Min(24),
            Constraint::Length(1),
        ]);

        Self {
            layout,
            ..Default::default()
        }
    }
}

impl View for KeyboardInner {
    fn on_event(
        &mut self,
        event: crossterm::event::KeyEvent,
        config: &mut Config,
    ) -> Option<crate::tui::Msg> {
        match event.code {
            KeyCode::Enter => {
                config.keyboard_layout = self.menu.current_item()?.to_string();
                self.menu.reset_search();

                Some(Msg::BackToMain)
            }
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
            let layouts = get_keyboard_layouts().wrap_err("Get keyboard layouts")?;

            self.menu.replace_items(layouts);
        });

        let chunks = self.layout.split(frame.size());

        self.menu
            .render_default(MenuArgs::builder().frame(frame).area(chunks[1]).build());

        self.menu.render_searchbar_default(frame, chunks[2]);

        Ok(())
    }
}
