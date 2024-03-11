use crossterm::cursor::SetCursorStyle;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{Paragraph, Widget},
};

use crate::{buffer::CursorMode, editor::Editor};

#[derive(Default)]
pub struct Viewport {
    pub width: usize,
    pub height: usize,
}

impl Viewport {
    pub fn update(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }
}

pub struct Cursor {
    pub x: u16,
    pub y: u16,
    pub mode: CursorMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventOutcome {
    Render,
    Ignore,
    Exit,
}

impl Cursor {
    pub fn style(&self) -> SetCursorStyle {
        match self.mode {
            CursorMode::Insert => SetCursorStyle::BlinkingBar,
            CursorMode::Normal | CursorMode::Visual => SetCursorStyle::BlinkingBlock,
        }
    }
}

pub struct Renderer<'a>(&'a Editor);

impl<'a> Renderer<'a> {
    pub fn new(editor: &'a Editor) -> Self {
        Self(editor)
    }

    #[inline]
    pub fn text(&self) -> Option<Text> {
        let buffer = self.0.workspace.current();

        let text = &buffer.text;
        let vscroll = buffer.vscroll;

        let start_byte = text.line_to_byte(vscroll);

        let end_index = vscroll + self.0.viewport.height - 1;
        let end_byte = text.line_to_byte(end_index.min(buffer.len_lines()));

        Some(Text::raw(text.slice(start_byte..end_byte)))
    }
}

impl<'a> Widget for Renderer<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.text() {
            Some(text) => {
                let inner = Paragraph::new(text);
                inner.render(area, buf);
            }
            None => log::warn!("nothing to render!"),
        }
    }
}
