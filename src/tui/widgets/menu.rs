use std::{
    collections::HashSet,
    ops::{Deref, DerefMut},
};

use crossterm::event::{KeyCode, KeyEvent};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{self, Block, Borders, List, ListItem, ListState},
    Frame,
};

use crate::tui::TuiCommand;

#[derive(Default)]
pub struct Menu {
    raw_items: Vec<String>,
    state: ListState,
    search_mode: bool,
    search_input: String,
    fuzzy_matcher: SkimMatcherV2,
    matched_items_count: Option<usize>,
}

impl Menu {
    pub const SEARCH_TIP: &'static str = r#"(Press "/" to search)"#;
    pub const NAVIGATION_TIP: &'static str = "j, k, Up, Down to move";

    pub fn new<I: Into<Vec<String>>>(items: I) -> Self {
        let raw_items: Vec<_> = items.into();
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            raw_items,
            state,
            ..Default::default()
        }
    }

    fn current_index(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn next_item(&mut self) {
        let cur = self.current_index();

        if cur.is_none() {
            return;
        }

        let cur = cur.unwrap();

        let mut next = cur + 1;

        if next > self.raw_items.len() - 1 {
            next = 0;
        }

        self.state.select(Some(next))
    }

    pub fn prev_item(&mut self) {
        let cur = self.current_index();

        if cur.is_none() {
            return;
        }

        let cur = cur.unwrap();

        let prev = if cur == 0 {
            self.raw_items.len() - 1
        } else {
            cur - 1
        };

        self.state.select(Some(prev))
    }

    fn items_count(&self) -> usize {
        if self.search_mode {
            0
        } else {
            self.raw_items.len()
        }
    }

    pub fn current_item(&self) -> Option<&str> {
        let cur = self.current_index()?;

        Some(&self.raw_items[cur])
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        self.render_with(frame, area, |items, selected_index| {
            let items = items
                .into_iter()
                .enumerate()
                .map(|(index, item)| {
                    let style = Style::default();

                    if index == selected_index {
                        item.style(style.fg(Color::White).bg(Color::LightBlue))
                    } else {
                        item.style(style)
                    }
                })
                .collect::<Vec<_>>();

            let list = widgets::List::new(items);

            list.block(Block::default().borders(Borders::ALL))
                .highlight_symbol("> ")
                .highlight_style(Style::default().fg(Color::White))
        });
    }

    pub fn render_with<B: Backend, F: Fn(Vec<ListItem>, usize) -> widgets::List>(
        &mut self,
        frame: &mut Frame<B>,
        area: Rect,
        f: F,
    ) {
        let items = if self.search_mode && !self.search_input.is_empty() {
            let items_match = self
                .raw_items
                .iter()
                .filter_map(|i| {
                    let indicies_set: HashSet<usize> = HashSet::from_iter(
                        self.fuzzy_matcher.fuzzy_indices(i, &self.search_input)?.1,
                    );

                    let mut spans: Vec<Span> = vec![];

                    let buf = i
                        .char_indices()
                        .fold(String::new(), |mut buf, (indice, c)| {
                            if !indicies_set.contains(&indice) {
                                buf.push(c);
                                return buf;
                            }

                            if !buf.is_empty() {
                                spans.push(Span::raw(buf));
                                buf = String::new()
                            }

                            spans.push(Span::styled(
                                c.to_string(),
                                Style::default().bg(Color::Yellow),
                            ));

                            buf
                        });

                    if !buf.is_empty() {
                        spans.push(Span::raw(buf))
                    }

                    Some(ListItem::new(Line::from(spans)))
                })
                .collect::<Vec<_>>();

            let matched_items_count = items_match.len();

            // self.update_state(matched_items_count, self.matched_items_count.unwrap_or(self.raw_items.len()));
            self.update_state(matched_items_count, self.raw_items.len());

            self.matched_items_count = Some(matched_items_count);

            items_match
        } else {
            self.raw_items.iter().map(|i| ListItem::new(&**i)).collect()
        };

        let cur = self.current_index();

        if self.current_index().is_none() {
            frame.render_widget(
                List::new([]).block(Block::default().borders(Borders::ALL)),
                area,
            );
            return;
        }

        let cur = cur.unwrap();

        let instance = f(items, cur);

        frame.render_stateful_widget(instance, area, &mut self.state)
    }

    pub fn update<F: FnOnce(&mut Vec<String>)>(&mut self, f: F) {
        let old_len = self.raw_items.len();

        f(&mut self.raw_items);

        self.update_state(self.raw_items.len(), old_len)
    }

    fn update_state(&mut self, len: usize, old_len: usize) {
        if old_len == len {
            return;
        }

        let new_selection = if len == 0 {
            None
        } else {
            Some(self.current_index().unwrap_or(0).min(len - 1))
        };

        // Reset offset and recalculate it when rendering
        *self.state.offset_mut() = 0;
        self.state.select(new_selection)
    }

    pub fn get_searchbar_text(&self) -> String {
        if !self.search_mode {
            Menu::SEARCH_TIP.to_string()
        } else {
            format!("/{}", self.search_input)
        }
    }
}

pub struct MenuView {
    inner: Menu,
}

impl MenuView {
    pub fn new<I: Into<Vec<String>>>(items: I) -> Self {
        let inner = Menu::new(items);

        Self { inner }
    }

    pub fn on_event(&mut self, event: KeyEvent) -> Option<TuiCommand> {
        match event.code {
            KeyCode::Up => {
                self.inner.prev_item();
                None
            }
            KeyCode::Char('k') => {
                if self.search_mode {
                    self.search_input.push('k');
                } else {
                    self.inner.prev_item();
                }

                None
            }
            KeyCode::Down => {
                self.inner.next_item();
                None
            }
            KeyCode::Char('j') => {
                if self.search_mode {
                    self.search_input.push('j');
                } else {
                    self.inner.next_item();
                }

                None
            }
            KeyCode::Char('/') => {
                if self.search_mode {
                    self.search_input.push('/')
                } else {
                    self.search_mode = true;
                }

                None
            }
            KeyCode::Char(c) => {
                if self.search_mode {
                    self.search_input.push(c)
                }

                None
            }
            KeyCode::Backspace => {
                if !self.search_mode {
                    return None;
                }

                if self.search_input.is_empty() {
                    self.search_mode = false;
                } else {
                    self.search_input.pop().unwrap();
                }

                None
            }
            _ => None,
        }
    }
}

impl Deref for MenuView {
    type Target = Menu;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for MenuView {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub fn to_string_vec<'a, A: IntoIterator<Item = &'a str>>(arr: A) -> Vec<String> {
    arr.into_iter().map(|s| s.to_string()).collect()
}
