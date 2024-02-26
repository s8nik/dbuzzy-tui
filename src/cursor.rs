use crossterm::cursor::SetCursorStyle;
use ropey::Rope;
use strum::EnumString;

#[derive(Debug, Default)]
pub struct Cursor {
    pub offset: usize,
    pub index: usize,
    pub vscroll: usize,
    pub mode: CursorMode,
}

impl Cursor {
    pub fn position(&self, text: &Rope) -> usize {
        let byte_index = text.line_to_byte(self.index);
        self.offset + byte_index
    }

    pub fn scroll(&mut self, max: usize) {
        let upper_bound = self.vscroll + max - 1;

        if self.index >= self.vscroll {
            self.vscroll = (self.vscroll + self.index).saturating_sub(upper_bound);
        } else if self.index < self.vscroll {
            self.vscroll = self.index;
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum CursorMode {
    Insert,
    #[default]
    Normal,
    Visual,
}

impl CursorMode {
    pub fn style(mode: CursorMode) -> SetCursorStyle {
        match mode {
            CursorMode::Insert => SetCursorStyle::BlinkingBar,
            CursorMode::Normal => SetCursorStyle::BlinkingBlock,
            CursorMode::Visual => SetCursorStyle::BlinkingBlock,
        }
    }
}
