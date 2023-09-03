use blanket::blanket;
use crossterm::event::KeyCode;

use macro_rules_attribute::derive;
use ratatui::{layout::Rect, text::Span, widgets::Paragraph, Frame};

use crate::{
    extensions::StrExt,
    tui::widgets::{selectable::delegate_selection_methods, WidgetEventHandler},
    LooseDefault,
};

use super::{
    searchable::{impl_search_methods, stylize_matched_item, SEARCH_TIP},
    Menu, MenuArgs,
};

#[blanket(derive(Rc))]
pub trait AsMenuItem {
    fn as_menu_item(&self) -> &str;
}

#[derive(Debug, LooseDefault!)]
pub struct CachedSearchableMenu<T> {
    inner: Menu,
    search_input: String,
    enable_search: bool,
    items: Vec<T>,
    matched_items_index: Vec<(usize, Vec<usize>)>,
    cached: bool,
}

impl<T> CachedSearchableMenu<T> {
    pub fn current_index(&self) -> Option<usize> {
        let mut index = self.inner.current_index()?;

        if self.is_searching() {
            index = self.matched_items_index[index].0
        }

        Some(index)
    }

    pub fn current_item(&self) -> Option<&T> {
        let index = self.current_index()?;
        Some(&self.items[index])
    }

    pub fn reset_search(&mut self) {
        self.enable_search = false;
        self.search_input.clear();
        self.cached = false
    }

    pub fn as_items(&self) -> &[T] {
        &self.items
    }

    pub fn update_items<F: FnOnce(&mut Vec<T>)>(&mut self, f: F) {
        f(&mut self.items);
        self.cached = false
    }

    pub fn replace_items<I: Into<Vec<T>>>(&mut self, items: I) {
        self.update_items(|i| *i = items.into())
    }
}

impl<T> CachedSearchableMenu<T> {
    delegate_selection_methods!(self.inner);
    impl_search_methods!();
}

impl<T> CachedSearchableMenu<T> {
    pub fn render_with<M: Fn(&T) -> &str>(&mut self, item_map_fn: M, args: MenuArgs) {
        let items = if self.is_searching() {
            if !self.cached {
                self.matched_items_index = self
                    .items
                    .iter()
                    .enumerate()
                    .filter_map(|(index, i)| {
                        let matched_indices = self.search_input.fuzzy_indices(item_map_fn(i))?;

                        Some((index, matched_indices))
                    })
                    .collect();
            }

            let matched_items = self
                .matched_items_index
                .iter()
                .map(|(index, matched_indices)| {
                    stylize_matched_item(item_map_fn(&self.items[*index]), matched_indices)
                })
                .collect::<Vec<_>>();

            matched_items
        } else {
            self.items
                .iter()
                .map(|i| vec![Span::raw(item_map_fn(i))])
                .collect()
        };

        self.inner.render_from(items, args)
    }
}

impl<T: AsMenuItem> CachedSearchableMenu<T> {
    pub fn render_default(&mut self, args: MenuArgs) {
        self.render_with(|item| T::as_menu_item(item), args)
    }
}

impl<T> WidgetEventHandler for CachedSearchableMenu<T> {
    fn on_event(&mut self, event: crossterm::event::KeyEvent) {
        macro_rules! impl_char_handler {
            ($c:literal $handler:ident) => {{
                if self.search_enabled() {
                    self.search_input.push($c);
                    self.cached = false;
                } else {
                    self.$handler();
                }
            }};
        }

        macro_rules! impl_fnkey_handler {
            ($handler:ident) => {{
                self.$handler();

                if self.is_searching() {
                    self.cached = true;
                }
            }};
        }

        match event.code {
            KeyCode::Up => impl_fnkey_handler!(select_prev_item),
            KeyCode::Down => impl_fnkey_handler!(select_next_item),
            KeyCode::Home => impl_fnkey_handler!(select_first_item),
            KeyCode::End => impl_fnkey_handler!(select_last_item),
            KeyCode::Char('k') => impl_char_handler!('k' select_prev_item),
            KeyCode::Char('j') => impl_char_handler!('j' select_next_item),
            KeyCode::Char('g') => impl_char_handler!('g' select_first_item),
            KeyCode::Char('G') => impl_char_handler!('G' select_last_item),
            KeyCode::Char('/') if !self.search_enabled() => self.enable_search = true,
            KeyCode::Char(c) => {
                if self.search_enabled() {
                    self.search_input.push(c);
                    self.cached = false
                }
            }
            KeyCode::Backspace => {
                if !self.search_enabled() {
                    return;
                }

                if self.search_input.is_empty() {
                    self.enable_search = false
                } else {
                    self.search_input.pop().unwrap();
                    self.cached = false;
                }
            }
            _ => {}
        }
    }
}

impl AsMenuItem for str {
    fn as_menu_item(&self) -> &str {
        self
    }
}

impl AsMenuItem for String {
    fn as_menu_item(&self) -> &str {
        self
    }
}
