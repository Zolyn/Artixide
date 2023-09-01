use color_eyre::Result;
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout};

use crate::{
    config::locale::LocaleConfig,
    fetch_data_if_needed,
    tui::{
        data::locale::get_locales,
        widgets::{
            menu::{CachedSearchableMenu, MenuArgs},
            Widget,
        },
        Msg, TuiBackend,
    },
    wrap_view,
};

use super::{horizontal_layout, vertical_layout, View};

wrap_view!(LocaleView, Locale);

#[derive(Debug, Default, Clone, Copy)]
enum Focus {
    #[default]
    Lang,
    Encoding,
}

impl Focus {
    fn switch(&mut self) {
        if let Self::Lang = self {
            *self = Self::Encoding
        } else {
            *self = Self::Lang
        }
    }
}

#[derive(Debug, Default)]
struct LocaleView {
    menus: [CachedSearchableMenu<String>; 2],
    v_layout: Layout,
    h_layout: Layout,
    focus: Focus,
}

macro_rules! get_menu_mut {
    ($self:ident) => {
        &mut $self.menus[$self.focus as usize]
    };
}

impl LocaleView {
    fn new() -> Self {
        let v_layout = vertical_layout([
            Constraint::Length(3),
            Constraint::Min(24),
            Constraint::Length(1),
        ]);

        let h_layout = horizontal_layout([
            Constraint::Percentage(49),
            Constraint::Percentage(2),
            Constraint::Percentage(49),
        ]);

        Self {
            v_layout,
            h_layout,
            ..Default::default()
        }
    }

    fn handle_select(&mut self, locale: &mut LocaleConfig) -> Option<Msg> {
        let menu = get_menu_mut!(self);
        let item = menu.current_item()?;

        let result = match self.focus {
            Focus::Lang => {
                let lang = if item.contains('@') {
                    let split = item.split('@').collect::<Vec<_>>();
                    assert_eq!(split.len(), 2, "Split length of item mismatch");

                    locale.modifier = Some(split[1].to_owned());
                    split[0]
                } else {
                    item.as_str()
                };

                let lang = if lang.contains('.') {
                    let split = lang.split('.').collect::<Vec<_>>();
                    assert_eq!(split.len(), 2, "Split length of lang mismatch");

                    split[0]
                } else {
                    lang
                };

                locale.lang = lang.to_owned();

                self.focus = Focus::Encoding;
                None
            }
            Focus::Encoding => {
                locale.encoding = item.to_string();
                self.focus = Focus::Lang;
                Some(Msg::BackToMain)
            }
        };

        menu.reset_search();
        result
    }
}

impl View for LocaleView {
    fn on_event(
        &mut self,
        event: crossterm::event::KeyEvent,
        config: &mut crate::config::Config,
    ) -> Option<crate::tui::Msg> {
        let menu = get_menu_mut!(self);

        match event.code {
            KeyCode::Tab => {
                self.focus.switch();
                None
            }
            KeyCode::Esc => {
                if menu.search_enabled() {
                    menu.reset_search();
                    return None;
                }

                Some(Msg::BackToMain)
            }
            KeyCode::Char('h') | KeyCode::Char('l') if !menu.search_enabled() => {
                self.focus.switch();
                None
            }
            KeyCode::Char('q') if !menu.search_enabled() => Some(Msg::BackToMain),
            KeyCode::Enter => self.handle_select(&mut config.locale),
            _ => menu.on_event(event),
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame<TuiBackend>) -> Result<()> {
        fetch_data_if_needed!({
            let (langs, encodings) = get_locales()?;

            let menu = &mut self.menus[Focus::Lang as usize];
            menu.replace_items(langs);

            let menu = &mut self.menus[Focus::Encoding as usize];
            menu.replace_items(encodings);
        });

        let v_chunks = self.v_layout.split(frame.size());

        let h_chunks = self.h_layout.split(v_chunks[1]);

        let [lang, enc] = &mut self.menus;

        let items = [
            (lang, Focus::Lang as usize, h_chunks[0]),
            (enc, Focus::Encoding as usize, h_chunks[2]),
        ];

        let cur = self.focus as usize;

        for (menu, tab, area) in items {
            let args_builder = MenuArgs::builder().frame(frame).area(area);

            if cur == tab {
                menu.render_default(args_builder.build())
            } else {
                menu.render_default(args_builder.hightlight_style(None).build())
            }
        }

        self.menus[self.focus as usize].render_searchbar_default(frame, v_chunks[2]);

        Ok(())
    }
}
