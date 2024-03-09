use crate::buffer::{Buffer, CursorMode};

pub(super) fn insert_mode(buffer: &mut Buffer) {
    buffer.update_cursor_mode(CursorMode::Insert);
}

pub(super) fn normal_mode(buffer: &mut Buffer) {
    buffer.update_cursor_mode(CursorMode::Normal);
}
