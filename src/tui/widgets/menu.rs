use derive_more::{Deref, DerefMut};
use ratatui::{
    style::Style,
    text::Line,
    widgets::{List, ListItem, ListState},
};

pub mod searchable;
pub mod searchable_cached;

use crate::extensions::StyleExt;

use super::{selectable::SelectableWidget, set_if_some, widget_args};
pub use searchable::SearchableMenu;
pub use searchable_cached::CachedSearchableMenu;

pub const NAVIGATION_TIP: &str = "Move: j, k, Up, Down, Home, End";

widget_args! {
    MenuArgs {
        #[builder(default = Some(Block::with_borders()))]
        block?: Block<'a>,
        #[builder(default = Some(Style::default().bg(Color::LightBlue).fg(Color::White)))]
        hightlight_style?: Style
    }
}

#[derive(Debug, Default, Deref, DerefMut)]
pub struct Menu(SelectableWidget<ListState>);

impl Menu {
    fn render_from_iter<'a, IntoIter: IntoIterator<Item = I>, I: Into<Line<'a>>>(
        &mut self,
        iter: IntoIter,
        len: usize,
        args: MenuArgs,
    ) {
        let MenuArgs {
            frame,
            area,
            block,
            hightlight_style,
        } = args;

        self.update_state(len);

        if len == 0 {
            let mut list = List::new([]);

            set_if_some!(list, block);

            frame.render_widget(list, area);
            return;
        }

        let cur = self.current_index().unwrap();

        let hightlight_style = hightlight_style.unwrap_or_default();

        let items = iter
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                let mut item: Line = item.into();

                item.patch_style(Style::with_fg());

                if index == cur {
                    for span in item.spans.iter_mut() {
                        span.style = span.style.patch(hightlight_style)
                    }
                }

                ListItem::new(item)
            })
            .collect::<Vec<_>>();

        let mut list = List::new(items).highlight_symbol("> ");

        set_if_some!(list, block);

        frame.render_stateful_widget(list, area, self.as_state_mut())
    }

    pub fn render(&mut self, items: &[&str], args: MenuArgs) {
        let len = items.len();
        self.render_from_iter(items.iter().copied(), len, args)
    }

    pub fn render_from<'a, V: Into<Vec<I>>, I: Into<Line<'a>>>(
        &mut self,
        items: V,
        args: MenuArgs,
    ) {
        let items: Vec<I> = items.into();
        let len = items.len();
        self.render_from_iter(items, len, args)
    }
}
