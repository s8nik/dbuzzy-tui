use crate::buffer::{Buffer, CursorMode};

pub(super) fn insert_mode(buffer: &mut Buffer) {
    buffer.update_cursor_mode(CursorMode::Insert);
}

pub(super) fn normal_mode(buffer: &mut Buffer) {
    buffer.update_cursor_mode(CursorMode::Normal);
}

pub(super) fn insert_mode_line_end(buffer: &mut Buffer) {
    buffer.update_offset(buffer.line_len_bytes());

    if buffer.index() < buffer.len_lines() - 1 {
        buffer.update_offset(buffer.offset() - 1);
    }

    buffer.update_cursor_mode(CursorMode::Insert);
}

pub(super) fn insert_mode_line_start(buffer: &mut Buffer) {
    buffer.update_offset(0);
    buffer.update_cursor_mode(CursorMode::Insert);
}

pub(super) fn insert_mode_line_next(buffer: &mut Buffer) {
    let index = buffer.index();
    let line_start_byte = buffer.text().line_to_byte(index + 1);
    buffer.text_mut().insert_char(line_start_byte, '\n');

    super::move_down(buffer);

    buffer.update_offset(0);
    buffer.update_cursor_mode(CursorMode::Insert);
}

pub(super) fn insert_mode_line_prev(buffer: &mut Buffer) {
    let index = buffer.index();
    let line_start_byte = buffer.text().line_to_byte(index);
    buffer.text_mut().insert_char(line_start_byte, '\n');

    buffer.update_offset(0);
    buffer.update_cursor_mode(CursorMode::Insert);
}
