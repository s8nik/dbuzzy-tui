use crossterm::cursor::SetCursorStyle;
use serde::Deserialize;

#[derive(Debug, Default, Deserialize, Clone, Copy, Eq, PartialEq, Hash)]
#[serde(rename_all = "lowercase")]
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
