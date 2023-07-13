use derive_setters::Setters;
use ratatui::{
    layout::Alignment,
    style::Style,
    widgets::{Block, BorderType, Borders},
};

use crate::set_if;

#[derive(Debug, Setters, Clone, Copy)]
#[setters(strip_option, borrow_self)]
pub struct BlockStyle {
    pub title_alignment: Option<Alignment>,
    #[setters(bool)]
    pub title_on_bottom: Option<bool>,
    pub borders: Option<Borders>,
    pub border_type: Option<BorderType>,
    pub border_style: Option<Style>,
    pub style: Option<Style>,
}

impl Default for BlockStyle {
    fn default() -> Self {
        Self {
            borders: Some(Borders::ALL),
            title_alignment: None,
            title_on_bottom: None,
            border_type: None,
            border_style: None,
            style: None,
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
