use crossterm::cursor::SetCursorStyle;

#[derive(Debug, Default, Clone, Copy)]
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
