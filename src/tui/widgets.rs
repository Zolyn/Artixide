use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, Borders},
};
use sealed::sealed;

use self::event::WidgetEventHandler;
use super::{
    views::{horizontal_layout, vertical_layout},
    Msg,
};

pub mod input;
pub mod menu;
mod selectable;
pub mod table;

// Referfences:
// https://users.rust-lang.org/t/how-to-match-a-optional-repetition-character-eg-with-input-when-defining-macro/64123
#[macro_export]
macro_rules! widget_args {
    (
        $name:ident $({
            $(
                $(#[$field_meta:meta])*
                $field_name:ident$(? $([$optional:tt])?)? : $field_ty:ty
            ),*
            $(,)*
        })*
    ) => {
        mod __widget_args {
            use typed_builder::TypedBuilder;
            use ratatui::{
                layout::*,
                widgets::*,
                style::*,
                Frame
            };

            use $crate::tui::{
                widgets::BlockExt,
                TuiBackend
            };

            #[derive(TypedBuilder)]
            pub struct $name<'a, 'b: 'a> {
                pub(super) frame: &'a mut Frame<'b, TuiBackend>,
                pub(super) area: Rect,
                $(
                    $(
                        $(#[$field_meta])*
                        pub(super) $field_name : $($($optional)?Option<)?$field_ty$($($optional)?>)?
                    ),*
                )*
            }
        }

        pub use __widget_args::$name;
    }
}

mod event {
    use crossterm::event::KeyEvent;

    use super::Widget;

    pub trait WidgetEventHandler {
        fn on_event(&mut self, event: KeyEvent);
    }

    impl<T: WidgetEventHandler> Widget for T {}
}

pub trait Widget: WidgetEventHandler {
    fn on_event(&mut self, event: KeyEvent) -> Option<Msg> {
        <Self as WidgetEventHandler>::on_event(self, event);
        None
    }
}

#[sealed]
pub trait BlockExt {
    fn with_borders() -> Block<'static> {
        Block::default()
            .borders(Borders::all())
            .style(Style::with_fg())
    }
}

#[sealed]
impl BlockExt for Block<'_> {}

#[sealed]
pub trait StyleExt {
    fn with_fg() -> Style {
        Style::default().fg(Color::Gray)
    }
}

#[sealed]
impl StyleExt for Style {}

pub fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let v_pack_size = (area.height - height) / 2;
    let h_pack_size = (area.width - width) / 2;

    let v_area = vertical_layout([
        Constraint::Length(v_pack_size),
        Constraint::Length(height),
        Constraint::Length(v_pack_size),
    ])
    .split(area)[1];

    horizontal_layout([
        Constraint::Length(h_pack_size),
        Constraint::Length(width),
        Constraint::Length(h_pack_size),
    ])
    .split(v_area)[1]
}

#[macro_export]
macro_rules! set_if_some {
    ($target:ident, $field:ident) => {
        if let Some(field) = $field {
            $target = $target.$field(field)
        };
    };
}
