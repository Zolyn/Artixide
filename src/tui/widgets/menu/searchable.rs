use crossterm::event::KeyCode;

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::Paragraph,
    Frame,
};

use super::{Menu, MenuArgs};
use crate::{
    extensions::StrExt,
    tui::widgets::{selectable::delegate_selection_methods, WidgetEventHandler},
};

pub const SEARCH_TIP: &str = r#"(Press "/" to search)"#;

macro_rules! impl_search_methods {
    () => {
        pub fn is_searching(&self) -> bool {
            self.enable_search && !self.search_input.is_empty()
        }

        pub fn search_enabled(&self) -> bool {
            self.enable_search
        }

        pub fn get_search_hint(&self) -> String {
            if self.enable_search {
                format!("/{}", self.search_input)
            } else {
                SEARCH_TIP.to_string()
            }
        }

        pub fn render_searchbar_default(
            &self,
            frame: &mut Frame<$crate::tui::TuiBackend>,
            area: Rect,
        ) {
            let searchbar = Paragraph::new(self.get_search_hint());

            frame.render_widget(searchbar, area)
        }
    };
}

pub(super) use impl_search_methods;

pub fn stylize_matched_item<'a, I: AsRef<str> + ?Sized>(
    item: &'a I,
    matched_indices: &[usize],
) -> Vec<Span<'a>> {
    let item = item.as_ref();
    let mut spans: Vec<Span> = vec![];

    let len = item.chars().count();
    let mut start = 0;
    let mut match_start: Option<usize> = None;
    let mut match_len: Option<usize> = None;

    for &index in matched_indices {
        assert!(
            start <= index,
            "start should be '<=' index while looping. \n Start: {}, Index: {}",
            start,
            index
        );

        if start < index {
            if let Some(match_start) = match_start.take() {
                let match_len = match_len.take().unwrap();

                spans.push(Span::styled(
                    item.slice(match_start, match_start + match_len).unwrap(),
                    Style::default().bg(Color::Yellow),
                ));
            }

            spans.push(Span::raw(item.slice(start, index).unwrap()));

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
            item.slice(match_start, match_start + match_len).unwrap(),
            Style::default().bg(Color::Yellow),
        ));
    }

    if start < len {
        spans.push(Span::raw(item.slice(start, len).unwrap()))
    }

    spans
}

#[derive(Debug, Default)]
pub struct SearchableMenu {
    inner: Menu,
    search_input: String,
    enable_search: bool,
    matched_items_index: Vec<usize>,
}

impl SearchableMenu {
    pub fn reset_search(&mut self) {
        self.enable_search = false;
        self.search_input.clear();
    }

    pub fn current_index(&self) -> Option<usize> {
        let mut index = self.inner.current_index()?;

        if self.is_searching() {
            index = self.matched_items_index[index]
        }

        Some(index)
    }

    pub fn inner_index(&self) -> Option<usize> {
        self.inner.current_index()
    }
}

impl SearchableMenu {
    delegate_selection_methods!(self.inner);
    impl_search_methods!();
}

impl SearchableMenu {
    pub fn render(&mut self, items: &[&str], args: MenuArgs) {
        let items = if self.is_searching() {
            let (index_list, spans): (Vec<_>, Vec<_>) = items
                .iter()
                .enumerate()
                .filter_map(|(index, i)| {
                    let matched_indices = self.search_input.fuzzy_indices(i)?;

                    let spans = stylize_matched_item(i, &matched_indices);

                    Some((index, spans))
                })
                .unzip();

            self.matched_items_index = index_list;

            spans
        } else {
            items.iter().map(|i| vec![Span::raw(*i)]).collect()
        };

        self.inner.render_from(items, args)
    }
}

impl WidgetEventHandler for SearchableMenu {
    fn on_event(&mut self, event: crossterm::event::KeyEvent) {
        macro_rules! impl_char_handler {
            ($c:literal $handler:ident) => {{
                if self.enable_search {
                    self.search_input.push($c);
                } else {
                    self.$handler();
                }
            }};
        }

        match event.code {
            KeyCode::Char('k') => impl_char_handler!('k' select_prev_item),
            KeyCode::Char('j') => impl_char_handler!('j' select_next_item),
            KeyCode::Char('g') => impl_char_handler!('g' select_first_item),
            KeyCode::Char('G') => impl_char_handler!('G' select_last_item),
            KeyCode::Char('/') if !self.enable_search => self.enable_search = true,
            KeyCode::Char(c) => {
                if self.enable_search {
                    self.search_input.push(c)
                }
            }
            KeyCode::Backspace => {
                if !self.enable_search {
                    return;
                }

                if self.search_input.is_empty() {
                    self.enable_search = false;
                } else {
                    self.search_input.pop().unwrap();
                }
            }
            _ => self.inner.on_event(event),
        };
    }
}
