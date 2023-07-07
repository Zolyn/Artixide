use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crossterm::event::{KeyCode, KeyEvent};
use fuzzy_matcher::FuzzyMatcher;
use ratatui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{self, Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::tui::{TuiCommand, FUZZY_MATCHER};

#[derive(Default)]
pub struct Menu {
    raw_items: Vec<Rc<String>>,
    state: RefCell<ListState>,
    search_mode: bool,
    search_input: String,
    matched_items: Vec<(Rc<String>, Vec<usize>)>,
    already_matched: bool,
    matched_items_count: Option<usize>,
}

impl Menu {
    pub const SEARCH_TIP: &'static str = r#"(Press "/" to search)"#;
    pub const NAVIGATION_TIP: &'static str = "j, k, Up, Down to move";

    pub fn new(items: Vec<String>) -> Self {
        let raw_items = items.into_iter().map(Rc::new).collect();
        let state = RefCell::new(ListState::default());
        state.borrow_mut().select(Some(0));

        Self {
            raw_items,
            state,
            ..Default::default()
        }
    }

    fn current_index(&self) -> Option<usize> {
        self.state.borrow().selected()
    }

    pub fn select_next_item(&self) {
        let cur = self.current_index();

        if cur.is_none() {
            return;
        }

        let cur = cur.unwrap();

        let mut next = cur + 1;

        if next > self.items_count() - 1 {
            next = 0;
        }

        self.state.borrow_mut().select(Some(next))
    }

    pub fn select_prev_item(&self) {
        let cur = self.current_index();

        if cur.is_none() {
            return;
        }

        let cur = cur.unwrap();

        let prev = if cur == 0 {
            self.items_count() - 1
        } else {
            cur - 1
        };

        self.state.borrow_mut().select(Some(prev))
    }

    pub fn select_first_item(&self) {
        if self.current_index().is_none() {
            return;
        }

        self.state.borrow_mut().select(Some(0))
    }

    pub fn select_last_item(&self) {
        if self.current_index().is_none() {
            return;
        }

        self.state.borrow_mut().select(Some(self.items_count() - 1))
    }

    fn items_count(&self) -> usize {
        if self.is_searching() {
            self.matched_items_count.unwrap()
        } else {
            self.raw_items.len()
        }
    }

    pub fn current_item(&self) -> Option<Rc<String>> {
        let cur = self.current_index()?;

        let item = if self.is_searching() {
            Rc::clone(&self.matched_items[cur].0)
        } else {
            Rc::clone(&self.raw_items[cur])
        };

        Some(item)
    }

    pub fn render<B: Backend>(&mut self, frame: &mut Frame<B>, area: Rect) {
        self.render_with(frame, area, |s| s.as_ref());
    }

    pub fn render_with<B: Backend, F: Fn(&Rc<String>) -> &I, I: AsRef<str> + ?Sized>(
        &mut self,
        frame: &mut Frame<B>,
        area: Rect,
        f: F,
    ) {
        let items = if self.is_searching() {
            if !self.already_matched {
                self.matched_items = self
                    .raw_items
                    .iter()
                    .filter_map(|i| {
                        let matched_indices = FUZZY_MATCHER
                            .with(|m| m.fuzzy_indices(i, &self.search_input))?
                            .1;

                        Some((Rc::clone(i), matched_indices))
                    })
                    .collect();
            }

            let matched_items = self
                .matched_items
                .iter()
                .map(|(item, matched_indices)| {
                    let item = f(item).as_ref();
                    let mut spans: Vec<Span> = vec![];

                    let len = item.chars().count();
                    let mut start = 0;
                    let mut match_start: Option<usize> = None;
                    let mut match_len: Option<usize> = None;

                    for &index in matched_indices {
                        if start > index {
                            unreachable!("start should always <= index while looping")
                        }

                        if start < index {
                            if let Some(match_start) = match_start.take() {
                                let match_len = match_len.take().unwrap();

                                spans.push(Span::styled(
                                    slice(item, match_start, match_start + match_len).unwrap(),
                                    Style::default().bg(Color::Yellow),
                                ));
                            }

                            spans.push(Span::raw(slice(item, start, index).unwrap()));

                            match_start = Some(index);
                            match_len = Some(1);
                            start = index + 1;
                            continue;
                        }

                        if match_start.is_some() {
                            *match_len.as_mut().unwrap() += 1;
                        } else {
                            assert_eq!(start, 0);
                            match_start = Some(0);
                            match_len = Some(1)
                        }

                        start = index + 1;
                    }

                    if let Some(match_start) = match_start.take() {
                        let match_len = match_len.take().unwrap();

                        spans.push(Span::styled(
                            slice(item, match_start, match_start + match_len).unwrap(),
                            Style::default().bg(Color::Yellow),
                        ));
                    }

                    if start < len {
                        spans.push(Span::raw(slice(item, start, len).unwrap()))
                    }

                    spans
                })
                .collect::<Vec<_>>();

            let matched_items_count = matched_items.len();

            self.update_state(
                matched_items_count,
                self.matched_items_count.unwrap_or(self.raw_items.len()),
            );

            self.matched_items_count = Some(matched_items_count);

            matched_items
        } else {
            let items = self
                .raw_items
                .iter()
                .map(|i| vec![Span::raw(f(i).as_ref())])
                .collect();

            // Update state if previous match has nothing
            // When there is nothing matched, the state will be None
            if self.search_mode && self.matched_items.is_empty() {
                self.update_state(self.raw_items.len(), 0)
            }

            items
        };

        let cur = self.current_index();

        if cur.is_none() {
            frame.render_widget(
                List::new([]).block(Block::default().borders(Borders::ALL)),
                area,
            );
            return;
        }

        let cur = cur.unwrap();

        let items = items
            .into_iter()
            .enumerate()
            .map(|(index, mut spans)| {
                if index == cur {
                    for span in spans.iter_mut() {
                        span.style.bg.get_or_insert(Color::LightBlue);
                        // FIXME: Upstream bug?
                        // Werid behavior in TTY when set color to white
                        // span.style.fg.insert(Color::White);
                    }
                }

                ListItem::new(Line::from(spans))
            })
            .collect::<Vec<_>>();

        let list = widgets::List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_symbol("> ")
            .highlight_style(Style::default());

        frame.render_stateful_widget(list, area, &mut self.state.borrow_mut())
    }

    pub fn update_items<F: FnOnce(&mut Vec<Rc<String>>)>(&mut self, f: F) {
        let old_len = self.raw_items.len();

        f(&mut self.raw_items);

        self.update_state(self.raw_items.len(), old_len)
    }

    pub fn replace_items_with(&mut self, items: Vec<String>) {
        let new_items = items.into_iter().map(Rc::new).collect();
        self.update_items(|i| *i = new_items)
    }

    pub fn replace_items(&mut self, items: Vec<Rc<String>>) {
        self.update_items(|i| *i = items)
    }

    fn update_state(&self, len: usize, old_len: usize) {
        if len == old_len {
            return;
        }

        let new_state = if len == 0 {
            None
        } else {
            Some(self.current_index().unwrap_or(0).min(len - 1))
        };

        let mut state = self.state.borrow_mut();

        // Reset offset and recalculate it when rendering
        *state.offset_mut() = 0;
        state.select(new_state)
    }

    pub fn get_searchbar_text(&self) -> String {
        if !self.search_mode {
            Menu::SEARCH_TIP.to_string()
        } else {
            format!("/{}", self.search_input)
        }
    }

    pub fn render_searchbar<B: Backend>(&self, frame: &mut Frame<B>, area: Rect) {
        let search_text = Line::from(vec![Span::raw(self.get_searchbar_text())]);

        let searchbar = Paragraph::new(search_text);

        frame.render_widget(searchbar, area)
    }

    pub fn search_mode(&self) -> bool {
        self.search_mode
    }

    fn is_searching(&self) -> bool {
        self.search_mode && !self.search_input.is_empty()
    }

    pub fn disable_search(&mut self) {
        self.search_mode = false;
        self.matched_items_count = None;
    }
}

pub struct MenuView {
    inner: Menu,
}

impl MenuView {
    pub fn new<I: Into<Vec<String>>>(items: I) -> Self {
        let inner = Menu::new(items.into());

        Self { inner }
    }

    pub fn on_event(&mut self, event: KeyEvent) -> Option<TuiCommand> {
        match event.code {
            KeyCode::Up => {
                self.inner.select_prev_item();

                if self.is_searching() {
                    self.already_matched = true;
                }

                None
            }
            KeyCode::Char('k') => {
                if self.search_mode {
                    self.search_input.push('k');
                    self.already_matched = false;
                } else {
                    self.inner.select_prev_item();
                }

                None
            }
            KeyCode::Down => {
                self.inner.select_next_item();

                if self.is_searching() {
                    self.already_matched = true;
                }

                None
            }
            KeyCode::Char('j') => {
                if self.search_mode {
                    self.search_input.push('j');
                    self.already_matched = false;
                } else {
                    self.inner.select_next_item();
                }

                None
            }
            KeyCode::Home => {
                self.inner.select_first_item();

                if self.is_searching() {
                    self.already_matched = true;
                }

                None
            }
            KeyCode::End => {
                self.inner.select_last_item();

                if self.is_searching() {
                    self.already_matched = true;
                }

                None
            }
            KeyCode::Char('/') => {
                if self.search_mode {
                    self.search_input.push('/');
                    self.already_matched = false
                } else {
                    self.search_mode = true;
                }

                None
            }
            KeyCode::Char(c) => {
                if self.search_mode {
                    self.search_input.push(c);
                    self.already_matched = false;
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
                    self.already_matched = false;

                    if self.search_input.is_empty() {
                        self.matched_items_count = None;
                    }
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

fn slice(s: &str, start: usize, end: usize) -> Option<&str> {
    if end < start {
        return None;
    }

    let start_index = s.char_indices().nth(start)?.0;

    if end == s.chars().count() {
        return Some(&s[start_index..]);
    }

    let end_index = start_index + s[start_index..].char_indices().nth(end - start)?.0;

    Some(&s[start_index..end_index])
}
