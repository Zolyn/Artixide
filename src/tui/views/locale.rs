use std::{collections::HashSet, fs};

use color_eyre::{eyre::eyre, Result};
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Layout},
    style::{Color, Style},
    widgets::Padding,
};
use regex::Regex;

use crate::{
    config::locale::LocaleConfig,
    tui::{widgets::menu::MenuView, TuiBackend, TuiCommand},
};

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

            Ok(caps
                .name("locale")
                .unwrap()
                .as_str()
                .trim_start_matches('#')
                .to_owned())
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
    config: LocaleConfig,
}

impl Locale {
    pub fn new() -> Self {
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

        let mut lang = MenuView::new([]);
        lang.title("Lang").padding(Padding::vertical(1));

        let mut encoding = MenuView::new([]);
        encoding.title("Encoding").padding(Padding::vertical(1));

        Self {
            menus: [lang, encoding],
            v_layout,
            h_layout,
            need_update: true,
            tab: LocaleTab::Lang,
            config: LocaleConfig::default(),
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
            KeyCode::Enter => {
                let menu = self.get_menu_mut();
                let item = menu.current_item()?;
                menu.disable_search();

                match self.tab {
                    LocaleTab::Lang => {
                        let lang = if item.contains('@') {
                            let split = item.split('@').collect::<Vec<_>>();
                            assert_eq!(split.len(), 2, "Split length of item mismatch");

                            self.config.modifier = split[1].to_owned();
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

                        self.config.lang = lang.to_owned();

                        self.tab = LocaleTab::Encoding;
                        None
                    }
                    LocaleTab::Encoding => {
                        self.config.encoding = item.to_string();
                        Some(TuiCommand::BackToMain)
                    }
                }
            }
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
        let focus_style = Style::default().fg(Color::LightBlue);

        match self.tab {
            LocaleTab::Lang => {
                lang.block_style.border_style(focus_style);

                enc.block_style.border_style.take();
            }
            LocaleTab::Encoding => {
                enc.block_style.border_style(focus_style);

                lang.block_style.border_style.take();
            }
        }

        lang.render(frame, h_chunks[0]);
        enc.render(frame, h_chunks[2]);

        self.menus[self.tab as usize].render_searchbar(frame, v_chunks[2]);

        Ok(())
    }
}
