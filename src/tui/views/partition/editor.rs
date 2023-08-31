use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{layout::Rect, widgets::Clear, Frame};

use crate::{
    let_irrefutable,
    tui::{
        data::partition::{Device, DiskSpace, MemPartition, MemTableEntry},
        widgets::{
            menu::{Menu, MenuArgs},
            Widget,
        },
        Msg, TuiBackend,
    },
};

use super::Focus;

#[derive(Debug)]
pub struct DiskEditor {
    items: Vec<&'static str>,
    menu: Menu,
    last_selected: Option<usize>,
}

impl DiskEditor {
    pub fn update_items(&mut self, entry_info: Option<(&[MemTableEntry], usize)>) {
        self.items.clear();
        self.last_selected.take();

        let Some((entries, selected)) = entry_info else {
            self.items.push("New partition table");
            return
        };

        let last_selected = Some(selected);

        if self.last_selected != last_selected {
            self.last_selected = last_selected
        }

        let entry = &entries[selected];

        match entry {
            MemTableEntry::Free(_) => self.items.push("Create partition"),
            MemTableEntry::Partition(_) => self.items.push("Delete partition"),
        };

        self.items.push("New partition table")
    }

    fn current_item(&self) -> Option<&str> {
        self.menu.current_index().map(|i| self.items[i])
    }

    pub(super) fn on_event(
        &mut self,
        event: KeyEvent,
        focus: &mut Focus,
        dev: &mut Device,
    ) -> Option<Msg> {
        match event.code {
            KeyCode::Tab => *focus = Focus::Table,
            KeyCode::Enter => match self.current_item()? {
                "Create partition" => {
                    let selected = self.last_selected.unwrap();

                    let_irrefutable!(dev, Device::Compatible(dev));
                    let_irrefutable!(&mut dev.mem_table[selected], MemTableEntry::Free(space));

                    let number = dev.number_pool.find_available_num()?;
                    let start = space.start();
                    let end = space.end();
                    let sectors = space.sectors();
                    let size = space.size();
                    let size_string = std::mem::take(space.size_string_mut());

                    dev.mem_table[selected] = MemTableEntry::Partition(
                        MemPartition::builder()
                            .number(number)
                            .start(start)
                            .end(end)
                            .sectors(sectors)
                            .size(size)
                            .size_string(size_string)
                            .build(),
                    );
                }
                "Delete partition" => {
                    let selected = self.last_selected.unwrap();

                    let_irrefutable!(dev, Device::Compatible(dev));

                    let is_prev_free = selected.checked_sub(1).is_some()
                        && matches!(
                            dev.mem_table.get(selected - 1),
                            Some(MemTableEntry::Free(_))
                        );

                    let is_next_free = selected + 1 < dev.mem_table.len()
                        && matches!(
                            dev.mem_table.get(selected.saturating_add(1)),
                            Some(MemTableEntry::Free(_))
                        );

                    match (is_prev_free, is_next_free) {
                        (true, true) => {
                            let removed = dev
                                .mem_table
                                .drain(selected..=selected + 1)
                                .map(|entry| match entry {
                                    MemTableEntry::Free(space) => space.as_raw_space(),
                                    MemTableEntry::Partition(part) => part.as_raw_space(),
                                })
                                .collect::<Vec<_>>();

                            assert!(removed.len() == 2);

                            let_irrefutable!(
                                &mut dev.mem_table[selected - 1],
                                MemTableEntry::Free(base)
                            );

                            for space in removed {
                                base.expand_right(space)
                            }
                        }
                        (true, false) => {
                            let_irrefutable!(
                                dev.mem_table.remove(selected),
                                MemTableEntry::Partition(part)
                            );
                            let_irrefutable!(
                                &mut dev.mem_table[selected - 1],
                                MemTableEntry::Free(base)
                            );

                            base.expand_right(part.as_raw_space())
                        }
                        (false, true) => {
                            let_irrefutable!(
                                dev.mem_table.remove(selected),
                                MemTableEntry::Partition(part)
                            );
                            let_irrefutable!(
                                &mut dev.mem_table[selected],
                                MemTableEntry::Free(base)
                            );

                            base.expand_left(part.as_raw_space())
                        }
                        (false, false) => {
                            let_irrefutable!(
                                &mut dev.mem_table[selected],
                                MemTableEntry::Partition(part)
                            );

                            let start = part.start();
                            let end = part.end();
                            let sectors = part.sectors();
                            let size = part.size();
                            let start_string = std::mem::take(part.start_string_mut());
                            let end_string = std::mem::take(part.end_string_mut());
                            let sectors_string = std::mem::take(part.sectors_string_mut());
                            let size_string = std::mem::take(part.size_string_mut());

                            let space = DiskSpace::builder()
                                .start(start)
                                .end(end)
                                .sectors(sectors)
                                .size(size)
                                .start_string(start_string)
                                .end_string(end_string)
                                .sectors_string(sectors_string)
                                .size_string(size_string)
                                .build();

                            dev.mem_table[selected] = MemTableEntry::Free(space)
                        }
                    }

                    dev.number_pool.set_unused(selected);
                }
                _ => unreachable!(),
            },
            _ => return self.menu.on_event(event),
        }

        None
    }

    pub fn render(&mut self, frame: &mut Frame<TuiBackend>, area: Rect, focus: bool) {
        frame.render_widget(Clear, area);

        let args_builder = MenuArgs::builder().frame(frame).area(area).block(None);

        if focus {
            self.menu.render(&self.items, args_builder.build())
        } else {
            self.menu
                .render(&self.items, args_builder.hightlight_style(None).build())
        }
    }
}

impl Default for DiskEditor {
    fn default() -> Self {
        Self {
            items: Vec::with_capacity(6),
            menu: Menu::default(),
            last_selected: None,
        }
    }
}
