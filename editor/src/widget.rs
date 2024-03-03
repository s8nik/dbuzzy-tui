use tui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{Paragraph, Widget},
};

use crate::workspace::Workspace;

pub struct EditorWidget<'a>(&'a Workspace<'a>);

impl<'a> EditorWidget<'a> {
    pub fn new(editor: &'a Workspace) -> Self {
        Self(editor)
    }

    #[inline]
    pub fn text(&self) -> Text {
        let buffer = self.0.current_buff();
        let content = buffer.content();

        let start_byte = content.text.line_to_byte(content.cursor.vscroll);

        let end_index = content.cursor.vscroll + self.0.viewport().1 - 1;
        let end_byte = content
            .text
            .line_to_byte(end_index.min(content.text.len_lines()));

        Text::raw(content.text.slice(start_byte..end_byte))
    }
}

impl<'a> Widget for EditorWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = Paragraph::new(self.text());
        inner.render(area, buf);
    }
}
