use tui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{Paragraph, Widget},
};

use crate::{buffer::Content, editor::Editor};

pub struct EditorWidget<'a>(&'a Editor<'a>);

impl<'a> EditorWidget<'a> {
    pub fn new(editor: &'a Editor) -> Self {
        Self(editor)
    }

    #[inline]
    pub fn text(&self) -> Text {
        let Content { text, cursor } = self.0.current_buff().content();
        let start_byte = text.line_to_byte(cursor.vscroll);

        let end_index = cursor.vscroll + self.0.viewport().1 - 1;
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
