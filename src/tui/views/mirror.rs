use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};

use crate::tui::{
    widgets::menu::{to_string_vec, MenuView},
    TuiBackend, TuiCommand,
};

use super::{vertical_layout, View};

struct MenuTab;

impl MenuTab {
    const MIRROR_TYPE: u8 = 0;
    const MIRROR_GROUP: u8 = 1;
    const SINGLE_MIRROR: u8 = 2;
}

pub struct Mirror {
    menus: [MenuView; 3],
    layout: Layout,
    tab: u8,
}

impl Mirror {
    const FIRST_MENU_ITEMS: [&str; 3] = ["Mirror group", "Single mirror", "aaa"];

    pub fn new() -> Self {
        let menus = [
            MenuView::new(to_string_vec(Self::FIRST_MENU_ITEMS)),
            MenuView::new([]),
            MenuView::new([]),
        ];

        let layout = vertical_layout([
            Constraint::Length(5),
            Constraint::Min(4),
            Constraint::Length(1),
        ]);

        Self {
            menus,
            layout,
            tab: MenuTab::MIRROR_TYPE,
        }
    }

    fn get_menu_mut(&mut self) -> &mut MenuView {
        &mut self.menus[self.tab as usize]
    }
}

impl View<TuiBackend> for Mirror {
    fn on_event(
        &mut self,
        event: crossterm::event::KeyEvent,
        _: &mut crate::config::Config,
    ) -> Option<crate::tui::TuiCommand> {
        let menu = self.get_menu_mut();

        match event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                if self.tab == 0 {
                    Some(TuiCommand::ChangeRoute('/'.to_string()))
                } else {
                    self.tab -= 1;
                    None
                }
            }
            _ => menu.on_event(event),
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame<TuiBackend>) -> anyhow::Result<()> {
        let chunks = self.layout.split(frame.size());
        let tab = self.tab;

        let menu: &mut MenuView = self.get_menu_mut();
        let menu_area: Rect;

        match tab {
            MenuTab::MIRROR_TYPE => {
                menu_area = vertical_layout([Constraint::Length(4), Constraint::Min(1)])
                    .split(chunks[1])[0];
            }
            _ => unimplemented!(),
        }

        menu.render(frame, menu_area);

        Ok(())
    }
}
