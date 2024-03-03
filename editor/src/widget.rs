use tui::{
    buffer::Buffer,
    layout::Rect,
    text::Text,
    widgets::{Paragraph, Widget},
};

use crate::editor::Editor;

pub struct EditorWidget<'a>(&'a Editor<'a>);

impl<'a> EditorWidget<'a> {
    pub fn new(editor: &'a Editor) -> Self {
        Self(editor)
    }

    #[inline]
    pub fn text(&self) -> Option<Text> {
        let workspace = self.0.workspace();

        let Some(buffer) = workspace.current_buff() else {
            return None;
        };

        let content = buffer.content();
        let start_byte = content.text.line_to_byte(content.cursor.vscroll);

        let end_index = content.cursor.vscroll + workspace.viewport().y - 1;
        let end_byte = content
            .text
            .line_to_byte(end_index.min(content.text.len_lines()));

        Some(Text::raw(content.text.slice(start_byte..end_byte)))
    }
}

impl<'a> Widget for EditorWidget<'a> {
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
