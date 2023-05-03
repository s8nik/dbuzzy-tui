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
        Text::raw(self.0.current_buff().text())
    }
}

impl<'a> Widget for EditorWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner = Paragraph::new(self.text());
        inner.render(area, buf);
    }
}
