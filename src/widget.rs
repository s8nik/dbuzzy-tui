use tui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{Paragraph, Widget},
};

use crate::editor::Editor;

pub struct EditorWidget<'a>(&'a Editor<'a>);

impl<'a> EditorWidget<'a> {
    pub fn new(editor: &'a Editor<'a>) -> Self {
        Self(editor)
    }

    #[inline]
    pub fn text(&self) -> Text {
        let buffer = self.0.current_buff();
        let text = buffer.text();
        let start_byte = text.line_to_byte(buffer.vscroll_index());

        let end_index = buffer.vscroll_index() + self.0.viewport().1 - 1;
        let end_byte = text.line_to_byte(end_index.min(text.len_lines()));

        Text::raw(text.slice(start_byte..end_byte))
    }
}

impl<'a> Widget for EditorWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = Paragraph::new(self.text());
        inner.render(area, buf);
    }
}
