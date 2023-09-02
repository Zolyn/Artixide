use std::fmt::Debug;

use color_eyre::Result;
use crossterm::event::KeyEvent;
use macro_rules_attribute::derive;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};
use strum::{EnumCount, EnumString};

use crate::config::Config;

use super::{route_map::RouteMap, Msg, TuiBackend};

#[derive(Debug, Clone, Copy, EnumCount, EnumString, RouteMap!)]
pub enum Route {
    Main,
    #[strum(serialize = "Keyboard layout")]
    Keyboard,
    Mirror,
    Locale,
    Timezone,
    Partition,
}

pub trait View: Debug {
    fn render(&mut self, frame: &mut Frame<TuiBackend>) -> Result<()>;
    fn on_event(&mut self, event: KeyEvent, config: &mut Config) -> Option<Msg>;
}

fn make_layout(constraints: Vec<Constraint>, direction: Direction) -> Layout {
    Layout::default()
        .direction(direction)
        .margin(0)
        .constraints(constraints)
}

pub fn vertical_layout<C: Into<Vec<Constraint>>>(constraints: C) -> Layout {
    make_layout(constraints.into(), Direction::Vertical)
}

pub fn horizontal_layout<C: Into<Vec<Constraint>>>(constraints: C) -> Layout {
    make_layout(constraints.into(), Direction::Horizontal)
}

macro_rules! fetch_data_if_needed {
    ($f:stmt) => {{
        use std::sync::atomic::{AtomicBool, Ordering};

        static NEED_FETCH: AtomicBool = AtomicBool::new(true);

        if NEED_FETCH.load(Ordering::Relaxed) {
            $f
            NEED_FETCH.store(false, Ordering::Relaxed);
        }
    }};
}

macro_rules! WrappedView {
    (
        $(#[$meta:meta])*
        struct $name:ident $rest:tt
    ) => {
        concat_idents::concat_idents!(struct_name = Wrapped, $name, {
            pub struct struct_name;

            impl struct_name {
                pub fn init() -> Box<dyn $crate::tui::views::View> {
                    $crate::assert_call_once!();

                    Box::new(<$name>::new())
                }
            }
        });
    };
}

pub(self) use {fetch_data_if_needed, WrappedView};
