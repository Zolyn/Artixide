use derive_setters::Setters;
use ratatui::{
    layout::Alignment,
    style::Style,
    widgets::{Block, BorderType, Borders},
};

use crate::set_if;

#[derive(Debug, Setters, Clone, Copy)]
#[setters(strip_option)]
pub struct BlockStyle {
    title_alignment: Option<Alignment>,
    #[setters(bool)]
    title_on_bottom: Option<bool>,
    borders: Option<Borders>,
    border_type: Option<BorderType>,
    border_style: Option<Style>,
    style: Option<Style>,
}

impl Default for BlockStyle {
    fn default() -> Self {
        Self {
            borders: Some(Borders::ALL),
            ..Default::default()
        }
    }
}

impl BlockStyle {
    pub fn build(self, mut block: Block) -> Block<'_> {
        set_if!(
            block,
            self,
            title_alignment,
            border_style,
            borders,
            border_type,
            style
        );

        if self.title_on_bottom.is_some() {
            block = block.title_on_bottom()
        }

        block
    }
}
