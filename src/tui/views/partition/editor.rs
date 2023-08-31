use std::str::FromStr;

use bytesize::ByteSize;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    widgets::{Block, Clear},
    Frame,
};

use crate::{
    let_irrefutable,
    string::StringExt,
    tui::{
        data::partition::{Device, DiskSpace, MemPartition, MemTableEntry},
        widgets::{
            input::{Input, InputCommand},
            menu::{Menu, MenuArgs},
            BlockExt, Widget,
        },
        Msg, TuiBackend,
    },
};

use super::Focus as ParentFocus;

const ERR_PARSE_SIZE: &str = "Failed to parse size";

#[derive(Debug)]
enum Focus {
    Menu,
    Create,
}

#[derive(Debug)]
pub struct DiskEditor {
    items: Vec<&'static str>,
    menu: Menu,
    input: Input,
    last_selected: Option<usize>,
    focus: Focus,
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

    fn handle_delete(&mut self, dev: &mut Device) {
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

                let_irrefutable!(&mut dev.mem_table[selected - 1], MemTableEntry::Free(base));

                for space in removed {
                    base.expand_right(space)
                }
            }
            (true, false) => {
                let_irrefutable!(
                    dev.mem_table.remove(selected),
                    MemTableEntry::Partition(part)
                );
                let_irrefutable!(&mut dev.mem_table[selected - 1], MemTableEntry::Free(base));

                base.expand_right(part.as_raw_space())
            }
            (false, true) => {
                let_irrefutable!(
                    dev.mem_table.remove(selected),
                    MemTableEntry::Partition(part)
                );
                let_irrefutable!(&mut dev.mem_table[selected], MemTableEntry::Free(base));

                base.expand_left(part.as_raw_space())
            }
            (false, false) => {
                let_irrefutable!(&mut dev.mem_table[selected], MemTableEntry::Partition(part));

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

    fn handle_menu(
        &mut self,
        event: KeyEvent,
        focus: &mut ParentFocus,
        dev: &mut Device,
    ) -> Option<Msg> {
        match event.code {
            KeyCode::Tab => *focus = ParentFocus::Table,
            KeyCode::Enter => match self.current_item()? {
                "Create partition" => self.focus = Focus::Create,
                "Delete partition" => self.handle_delete(dev),
                _ => unreachable!(),
            },
            _ => return self.menu.on_event(event),
        }

        None
    }

    fn handle_create(&mut self, event: KeyEvent, dev: &mut Device) -> Option<Msg> {
        let command = self.input.on_event(event)?;

        if matches!(command, InputCommand::Cancel) {
            self.input.clear();
            self.focus = Focus::Menu;
            return None;
        }

        let selected = self.last_selected.unwrap();

        let_irrefutable!(dev, Device::Compatible(dev));
        let_irrefutable!(&mut dev.mem_table[selected], MemTableEntry::Free(space));

        let sector_size = *dev.disk.sector_size();

        let input = self.input.as_str().trim();
        let (last_char_index, c) = input.char_indices().last().unwrap();

        let sectors = if c.eq_ignore_ascii_case(&'S') {
            input[..last_char_index].trim().parse::<u64>().unwrap()
        } else {
            let Ok(ByteSize(size)) = ByteSize::from_str(input) else { todo!() };
            size / sector_size as u64
        };

        let free_sectors = space.sectors;

        // TODO: Alignment
        assert!(sectors > 0 && sectors <= free_sectors);

        let remaining = free_sectors - sectors;
        let has_remaining = remaining > 0;

        let number = dev.number_pool.find_available_num()?;
        let start = space.start;
        let start_string = space.start_string.take();
        let end: u64;

        let part = if has_remaining {
            end = start + sectors - 1;
            let size = sectors * sector_size as u64;

            MemPartition::builder()
                .number(number)
                .start(start)
                .end(end)
                .size(size)
                .sectors(sectors)
                .start_string(start_string)
                .build()
        } else {
            end = space.end;
            let size = space.size;
            let size_string = space.size_string.take();
            let end_string = space.end_string.take();
            let sectors_string = space.sectors_string.take();

            MemPartition::builder()
                .number(number)
                .start(start)
                .end(end)
                .size(size)
                .sectors(sectors)
                .start_string(start_string)
                .end_string(end_string)
                .sectors_string(sectors_string)
                .size_string(size_string)
                .build()
        };

        if has_remaining {
            let start = end + 1;
            let end = space.end;
            let sectors = remaining;
            let size = remaining * sector_size as u64;
            let end_string = space.end_string.take();

            let space = DiskSpace::builder()
                .start(start)
                .end(end)
                .sectors(sectors)
                .size(size)
                .end_string(end_string)
                .build();

            dev.mem_table
                .insert(selected + 1, MemTableEntry::Free(space))
        }

        dev.mem_table[selected] = MemTableEntry::Partition(part);

        self.focus = Focus::Menu;
        None
    }

    pub(super) fn on_event(
        &mut self,
        event: KeyEvent,
        focus: &mut ParentFocus,
        dev: &mut Device,
    ) -> Option<Msg> {
        match self.focus {
            Focus::Menu => self.handle_menu(event, focus, dev),
            Focus::Create => self.handle_create(event, dev),
        }
    }

    fn render_create_menu(&mut self, frame: &mut Frame<TuiBackend>, area: Rect) {
        let block = Block::with_borders().title("Partition size (S for sectors)");

        let inner = block.inner(area);

        frame.render_widget(block, area);

        self.input.render(frame, inner)
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

        if matches!(self.focus, Focus::Create) {
            let area = Rect {
                x: area.x,
                y: area.y - 3,
                height: 3,
                width: area.width,
            };

            self.render_create_menu(frame, area)
        }
    }
}

impl Default for DiskEditor {
    fn default() -> Self {
        Self {
            items: Vec::with_capacity(6),
            menu: Menu::default(),
            input: Input::default(),
            last_selected: None,
            focus: Focus::Menu,
        }
    }
}
