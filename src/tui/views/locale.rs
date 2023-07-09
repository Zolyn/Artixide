use std::{collections::HashSet, fs};

use color_eyre::{eyre::eyre, Result};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
};
use regex::Regex;

use crate::tui::{widgets::menu::MenuView, TuiBackend, TuiCommand};

use super::{horizontal_layout, vertical_layout, View};

fn get_locales() -> Result<(Vec<String>, Vec<String>)> {
    let locale_re =
        Regex::new(r"^(?<locale>#?[a-z]+(_[A-Z]+)?(\@[a-z]+)?(\.[^\s]+)?)\s(?<encoding>[^\s]+)")
            .unwrap();
    let locale_gen = fs::read_to_string("/etc/locale.gen")?;

    let mut encoding_set: HashSet<&str> = HashSet::new();

    let langs = locale_gen
        .lines()
        .enumerate()
        .skip_while(|(_, line)| !locale_re.is_match(line))
        .map(|(i, line)| {
            let caps = locale_re
                .captures(line)
                .ok_or_else(|| eyre!("Failed to match locale: {}(line {})", line, i))?;

            encoding_set.insert(caps.name("encoding").unwrap().as_str());

            Ok(caps.name("locale").unwrap().as_str().to_owned())
        })
        .collect::<Result<Vec<_>>>()?;

    let mut encodings = encoding_set
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();

    encodings.sort();

    Ok((langs, encodings))
}

#[derive(Debug, Clone, Copy)]
enum LocaleTab {
    Lang,
    Encoding,
}

pub struct Locale {
    menus: [MenuView; 2],
    v_layout: Layout,
    h_layout: Layout,
    need_update: bool,
    tab: LocaleTab,
}

impl Locale {
    pub fn new() -> Self {
        let v_layout = vertical_layout([
            Constraint::Length(3),
            Constraint::Min(24),
            Constraint::Length(1),
        ]);

        let h_layout = horizontal_layout([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)]);

        let menus = [MenuView::new([]), MenuView::new([])];

        Self {
            menus,
            v_layout,
            h_layout,
            need_update: true,
            tab: LocaleTab::Lang,
        }
    }

    fn get_menu_mut(&mut self) -> &mut MenuView {
        &mut self.menus[self.tab as usize]
    }
}

impl View<TuiBackend> for Locale {
    fn on_event(
        &mut self,
        event: crossterm::event::KeyEvent,
        _config: &mut crate::config::Config,
    ) -> Option<crate::tui::TuiCommand> {
        let menu = self.get_menu_mut();

        match event.code {
            KeyCode::Tab => {
                let tab = match self.tab {
                    LocaleTab::Lang => LocaleTab::Encoding,
                    LocaleTab::Encoding => LocaleTab::Lang,
                };

                self.tab = tab;

                None
            }
            KeyCode::Char('q') if !menu.search_mode() => Some(TuiCommand::BackToMain),
            _ => menu.on_event(event),
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame<TuiBackend>) -> Result<()> {
        let v_chunks = self.v_layout.split(frame.size());

        if self.need_update {
            let (langs, encodings) = get_locales()?;

            let menu = &mut self.menus[LocaleTab::Lang as usize];

            menu.replace_items_with(langs);

            let menu = &mut self.menus[LocaleTab::Encoding as usize];

            menu.replace_items_with(encodings);

            self.need_update = false;
        }

        let h_chunks = self.h_layout.split(v_chunks[1]);

        let [lang, enc] = &mut self.menus;
        let focus_style = Style::default().fg(Color::White);

        match self.tab {
            LocaleTab::Lang => {
                lang.border_style = Some(focus_style);

                enc.border_style.take();
            }
            LocaleTab::Encoding => {
                enc.border_style = Some(focus_style);

                lang.border_style.take();
            }
        }

        lang.render(frame, h_chunks[0]);
        enc.render(frame, h_chunks[1]);

        self.menus[self.tab as usize].render_searchbar(frame, v_chunks[2]);

        Ok(())
    }
}
