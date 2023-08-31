use std::fmt::Debug;

use color_eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::config::Config;

use super::{Msg, TuiBackend};

pub mod keyboard;
pub mod locale;
pub mod main;
pub mod mirror;
pub mod partition;
pub mod timezone;

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

#[macro_export]
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

#[macro_export]
macro_rules! assert_call_once {
    () => {{
        use std::sync::atomic::{AtomicBool, Ordering};

        static CALLED: AtomicBool = AtomicBool::new(false);

        let called = CALLED.swap(true, Ordering::Relaxed);
        assert!(called == false, "View can be only created once")
    }};
}

#[macro_export]
macro_rules! wrap_view {
    ($inner:ty, $name:ident) => {
        #[derive(Debug)]
        pub struct $name;

        impl $name {
            pub fn init() -> Box<dyn $crate::tui::views::View> {
                $crate::assert_call_once!();

                Box::new(<$inner>::new())
            }
        }
    };
}

#[macro_export]
macro_rules! let_irrefutable {
    ($v:expr, $p:pat) => {
        let $p = $v else { unreachable!() };
    };
}

#[macro_export]
macro_rules! match_irrefutable {
    ($v:expr, $p:pat, $ret:expr) => {{
        match $v {
            $p => $ret,
            _ => unreachable!(),
        }
    }};
}
