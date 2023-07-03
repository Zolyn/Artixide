use std::{
    collections::HashSet,
    ops::{Deref, DerefMut, Range},
};

use crossterm::event::{KeyCode, KeyEvent};
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};
use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{self, Block, Borders, ListItem, ListState},
    Frame,
};

use crate::tui::TuiCommand;

#[derive(Default)]
pub struct Menu {
    raw_items: Vec<String>,
    index_range: Range<usize>,
    state: ListState,
    reset_when_update: bool,
    search_mode: bool,
    search_input: String,
    fuzzy_matcher: SkimMatcherV2,
}

impl Menu {
    pub const SEARCH_TIP: &'static str = r#"(Press "/" to search)"#;
    pub const NAVIGATION_TIP: &'static str = "j, k, Up, Down to move";

    pub fn new<I: Into<Vec<String>>>(items: I) -> Self {
        let raw_items: Vec<_> = items.into();
        let index_range = 0..raw_items.len();
        let mut state = ListState::default();
        state.select(Some(0));

        Self {
            raw_items,
            index_range,
            state,
            ..Default::default()
        }
    }

    fn current_index(&self) -> usize {
        self.state.selected().unwrap()
    }

    pub fn reset_when_update(mut self, val: bool) -> Self {
        self.reset_when_update = val;
        self
    }

    pub fn next_item(&mut self) {
        let mut next = self.current_index() + 1;

        if !self.index_range.contains(&next) {
            next = 0;
        }

        self.state.select(Some(next))
    }

    pub fn prev_item(&mut self) {
        let cur = self.current_index();
        let prev = if cur == 0 {
            self.raw_items.len() - 1
        } else {
            cur - 1
        };

        self.state.select(Some(prev))
    }

    pub fn current_item(&self) -> &str {
        &self.raw_items[self.current_index()]
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        let selected_index = self.current_index();

        self.render_with(frame, area, |items| {
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

    pub fn render_with<B: Backend, F: Fn(Vec<ListItem>) -> widgets::List>(
        &mut self,
        frame: &mut Frame<B>,
        area: Rect,
        f: F,
    ) {
        let items = if self.search_mode && !self.search_input.is_empty() {
            self.raw_items
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
                .collect::<Vec<_>>()
        } else {
            self.raw_items.iter().map(|i| ListItem::new(&**i)).collect()
        };

        let instance = f(items);

        frame.render_stateful_widget(instance, area, &mut self.state)
    }
    pub fn update<F: FnOnce(&mut Vec<String>)>(&mut self, f: F) {
        let old_len = self.raw_items.len();

        f(&mut self.raw_items);

        let new_len = self.raw_items.len();

        if old_len == new_len {
            return;
        }

        self.index_range = 0..new_len;

        let new_selection = if self.reset_when_update {
            0
        } else {
            self.state.selected().unwrap().min(new_len - 1)
        };

        self.state.select(Some(new_selection))
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
