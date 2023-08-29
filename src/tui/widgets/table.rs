use derive_more::{Deref, DerefMut};
use ratatui::widgets::{Row, Table as RawTable, TableState};

use crate::widget_args;

use super::selectable::SelectableWidget;

widget_args! {
    TableArgs {
        #[builder(default, setter(strip_option))]
        header?: Row<'a>,
        #[builder(default, setter(strip_option))]
        column_spacing?: u16,
        #[builder(default = Some(Style::default().fg(Color::Black).bg(Color::Gray)))]
        highlight_style?: Style,
        widths: &'a [Constraint],
        #[builder(default = Block::with_borders())]
        block: Block<'a>,
    }
}

#[derive(Debug, Default, Deref, DerefMut)]
pub struct Table(SelectableWidget<TableState>);

impl Table {
    pub fn render<'a, V: Into<Vec<Row<'a>>>>(&mut self, rows: V, args: TableArgs) {
        let TableArgs {
            frame,
            area,
            block,
            header,
            widths,
            column_spacing,
            highlight_style,
        } = args;
        let rows: Vec<Row> = rows.into();

        let len = rows.len();
        self.update_state(len);

        if len == 0 {
            frame.render_widget(RawTable::new([]).block(block), area);
            return;
        }

        let hightlight_style = highlight_style.unwrap_or_default();

        let mut table = RawTable::new(rows)
            .block(block)
            .widths(widths)
            .highlight_style(hightlight_style)
            .highlight_symbol(">> ");

        if let Some(header) = header {
            table = table.header(header)
        }

        if let Some(spacing) = column_spacing {
            table = table.column_spacing(spacing)
        }

        frame.render_stateful_widget(table, area, self.as_state_mut())
    }
}
