use color_eyre::eyre::bail;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Cell, Row},
    Frame,
};

use self::editor::DiskEditor;
use crate::{
    config::Config,
    fetch_data_if_needed, lazy, let_irrefutable,
    tui::{
        data::partition::{get_devices, Device, DiskSpace, MemPartition, MemTableEntry},
        widgets::{
            table::{Table, TableArgs},
            Widget,
        },
        Msg, TuiBackend,
    },
    wrap_view,
};

use super::{vertical_layout, View};

mod editor;

wrap_view!(PartitionView, Partition);

lazy! {
    static LAYOUT: Layout = vertical_layout([
        Constraint::Length(5),
        Constraint::Max(20),
        Constraint::Min(1),
    ])
}

const HEADER: [&str; 9] = [
    "Name",
    "ESP",
    "Start",
    "End",
    "Sectors",
    "Filesystem",
    "Filesystem label",
    "Mountpoint",
    "Size",
];

const TABLE_WIDTHS: &[Constraint] = &[
    Constraint::Percentage(10),
    Constraint::Length(5),
    Constraint::Percentage(10),
    Constraint::Percentage(10),
    Constraint::Percentage(10),
    Constraint::Length(10),
    Constraint::Percentage(10),
    Constraint::Percentage(10),
    Constraint::Percentage(10),
];

#[derive(Debug, Clone, Copy, Default)]
enum Focus {
    #[default]
    Table,
    Editor,
}

#[derive(Debug, Default)]
struct PartitionView {
    devices: Vec<Device>,
    current_device: usize,
    table: Table,
    focus: Focus,
    editor: DiskEditor,
}

impl PartitionView {
    fn new() -> Self {
        Self {
            current_device: 0,
            ..Default::default()
        }
    }

    fn make_partition_row<'a>(parent: &'a str, part: &'a MemPartition) -> Row<'a> {
        let esp = if *part.bootable() { "*" } else { "" };
        let label = match part.label() {
            Some(l) => l.as_str(),
            None => "",
        };

        let mountpoint = match part.mountpoint() {
            Some(m) => m.as_str(),
            None => "",
        };

        Row::new([
            Cell::from(Line::from(vec![
                Span::from(parent),
                Span::from(part.number_string().as_str()),
            ])),
            Cell::from(esp),
            Cell::from(part.start_string().as_str()),
            Cell::from(part.end_string().as_str()),
            Cell::from(part.sectors_string().as_str()),
            Cell::from(part.filesystem().as_ref()),
            Cell::from(label),
            Cell::from(mountpoint),
            Cell::from(part.size_string().as_str()),
        ])
    }

    fn make_free_space_row(space: &DiskSpace) -> Row {
        Row::new([
            "Free space",
            "",
            space.start_string().as_str(),
            space.end_string().as_str(),
            space.sectors_string().as_str(),
            "",
            "",
            "",
            space.size_string().as_str(),
        ])
        .style(Style::default().fg(Color::Green))
    }

    fn get_device(&self) -> &Device {
        &self.devices[self.current_device]
    }
}

macro_rules! get_device_mut {
    ($self:ident) => {
        &mut $self.devices[$self.current_device]
    };
}

impl PartitionView {
    fn render_table(&mut self, frame: &mut Frame<TuiBackend>, area: Rect) {
        let focus = matches!(self.focus, Focus::Table);

        let (rows, header) = match &self.devices[self.current_device] {
            Device::Compatible(dev) => {
                let parent = dev.disk.path().as_str();

                let rows = dev
                    .mem_table
                    .iter()
                    .map(|part| match part {
                        MemTableEntry::Partition(part) => Self::make_partition_row(parent, part),
                        MemTableEntry::Free(space) => Self::make_free_space_row(space),
                    })
                    .collect::<Vec<_>>();

                let header = {
                    let mut header = HEADER;

                    if *dev.disk.is_gpt() {
                        header[1] = "Boot"
                    }

                    header
                };

                (rows, header)
            }
            Device::Incompatible(_) => (vec![], HEADER),
        };

        let args_builder = TableArgs::builder()
            .frame(frame)
            .area(area)
            .header(Row::new(header))
            .widths(TABLE_WIDTHS)
            .column_spacing(3);

        if focus {
            self.table.render(rows, args_builder.build())
        } else {
            self.table
                .render(rows, args_builder.highlight_style(None).build())
        }
    }
}

impl PartitionView {
    fn handle_table(&mut self, event: KeyEvent, _config: &mut Config) -> Option<Msg> {
        match event.code {
            KeyCode::Char('q') => Some(Msg::BackToMain),
            KeyCode::Tab => {
                self.focus = Focus::Editor;
                None
            }
            _ => self.table.on_event(event),
        }
    }
}

impl View for PartitionView {
    fn on_event(
        &mut self,
        event: crossterm::event::KeyEvent,
        config: &mut crate::config::Config,
    ) -> Option<crate::tui::Msg> {
        match self.focus {
            Focus::Table => self.handle_table(event, config),
            Focus::Editor => self.editor.on_event(
                event,
                &mut self.focus,
                &mut self.devices[self.current_device],
            ),
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame<TuiBackend>) -> color_eyre::Result<()> {
        fetch_data_if_needed!({
            self.devices = get_devices()?;

            if self.devices.is_empty() {
                bail!("No devices found")
            }
        });

        let chunks = LAYOUT.split(frame.size());

        self.render_table(frame, chunks[1]);

        let entry_info = self.table.current_index().map(|i| {
            let_irrefutable!(&self.devices[self.current_device], Device::Compatible(dev));
            (dev.mem_table.as_slice(), i)
        });

        self.editor.update_items(entry_info);

        self.editor
            .render(frame, chunks[2], matches!(self.focus, Focus::Editor));

        Ok(())
    }
}
