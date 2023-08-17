use crossterm::cursor::SetCursorStyle;
use strum::EnumString;

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum CursorMode {
    Insert,
    #[default]
    Normal,
    Visual,
}

impl CursorMode {
    pub fn cursor_style(mode: CursorMode) -> SetCursorStyle {
        match mode {
            CursorMode::Insert => SetCursorStyle::BlinkingBar,
            CursorMode::Normal => SetCursorStyle::BlinkingBlock,
            CursorMode::Visual => SetCursorStyle::BlinkingBlock,
        }
    }
}
