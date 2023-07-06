use std::{fs, mem};

use anyhow::Result;
use crossterm::event::KeyCode;
use ratatui::layout::{Constraint, Layout, Rect};

use crate::tui::{
    widgets::menu::{to_string_vec, MenuView},
    TuiBackend, TuiCommand,
};

use super::{vertical_layout, View};

fn get_grouped_mirrors() -> Result<Vec<(String, Vec<String>)>> {
    let mirror_list = fs::read_to_string("/etc/pacman.d/mirrorlist")?;

    let mut group = "Default mirrors".to_string();
    let mut servers = vec![];
    let mut result = vec![];

    for line in mirror_list
        .lines()
        .skip_while(|line| !line.contains("Default mirrors"))
        .skip(1)
    {
        if line.starts_with("Server") {
            servers.push(line.trim_start_matches("Server = ").to_owned())
        } else if line.starts_with("# ") {
            result.push((
                mem::replace(&mut group, line.trim_start_matches("# ").to_owned()),
                mem::take(&mut servers),
            ))
        }
    }

    result.push((group, servers));

    Ok(result)
}

#[derive(Debug, Clone, Copy)]
enum MenuTab {
    MirrorType,
    MirrorGroup,
    SingleMirror,
}

pub struct Mirror {
    menus: [MenuView; 3],
    layout: Layout,
    tab: MenuTab,
    need_update: bool,
}

impl Mirror {
    const MIRROR_TYPES: [&str; 2] = ["Mirror group", "Single mirror"];

    pub fn new() -> Self {
        let menus = [
            MenuView::new(to_string_vec(Self::MIRROR_TYPES)),
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
            tab: MenuTab::MirrorType,
            need_update: true,
        }
    }

    fn get_menu_mut(&mut self) -> &mut MenuView {
        &mut self.menus[self.tab as usize]
    }

    fn current_item(&self) -> Option<&str> {
        self.menus[self.tab as usize].current_item()
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
            KeyCode::Esc | KeyCode::Char('q') => match self.tab {
                MenuTab::MirrorGroup | MenuTab::SingleMirror => {
                    self.tab = MenuTab::MirrorType;
                    None
                }
                MenuTab::MirrorType => Some(TuiCommand::ChangeRoute("/".to_string())),
            },
            KeyCode::Enter => {
                let cur = self.current_item()?;

                match self.tab {
                    MenuTab::MirrorType => match cur {
                        "Mirror group" => self.tab = MenuTab::MirrorGroup,
                        "Single mirror" => self.tab = MenuTab::SingleMirror,
                        _ => unreachable!(),
                    },
                    _ => unimplemented!(),
                };

                None
            }
            _ => menu.on_event(event),
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame<TuiBackend>) -> anyhow::Result<()> {
        let chunks = self.layout.split(frame.size());

        if self.need_update {
            let (group, servers): (Vec<_>, Vec<_>) = get_grouped_mirrors()?.into_iter().unzip();

            let servers = servers.into_iter().flatten().collect::<Vec<_>>();

            let menu = &mut self.menus[MenuTab::MirrorGroup as usize];
            menu.replace_items(group);

            let menu = &mut self.menus[MenuTab::SingleMirror as usize];
            menu.replace_items(servers);

            self.need_update = false;
        }

        let mut menu_area: Rect = chunks[1];

        match self.tab {
            MenuTab::MirrorType => {
                menu_area = vertical_layout([Constraint::Length(4), Constraint::Min(1)])
                    .split(chunks[1])[0];
            }
            _ => {}
        }

        let menu: &mut MenuView = self.get_menu_mut();

        menu.render(frame, menu_area);

        menu.render_searchbar(frame, chunks[2]);

        Ok(())
    }
}
