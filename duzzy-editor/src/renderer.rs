use crossterm::cursor::SetCursorStyle;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{Paragraph, Widget},
};

use crate::{buffer::Mode, editor::DuzzyEditor};

#[derive(Default)]
pub(super) struct Viewport {
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
    pub mode: Mode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventOutcome {
    Render,
    Ignore,
    Exit,
}

impl Cursor {
    pub const fn style(&self) -> SetCursorStyle {
        match self.mode {
            Mode::Insert => SetCursorStyle::BlinkingBar,
            Mode::Normal | Mode::Visual(_) => SetCursorStyle::BlinkingBlock,
        }
    }
}

pub struct Renderer<'a>(&'a DuzzyEditor);

impl<'a> Renderer<'a> {
    pub const fn new(editor: &'a DuzzyEditor) -> Self {
        Self(editor)
    }

    #[inline]
    pub fn text(&self) -> Option<Text> {
        let buf = self.0.workspace.curr().buf();

        let text = buf.text();
        let vscroll = buf.vscroll();

        let start_byte = text.line_to_byte(vscroll);

        let end_index = vscroll + self.0.viewport().1 - 1;
        let end_byte = text.line_to_byte(end_index.min(buf.len_lines()));

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
