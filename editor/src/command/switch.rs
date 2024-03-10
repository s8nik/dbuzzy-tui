use crate::buffer::{Buffer, CursorMode};

pub(super) fn insert_mode(buffer: &mut Buffer) {
    buffer.update_cursor_mode(CursorMode::Insert).ok();
}

pub(super) fn normal_mode(buffer: &mut Buffer) {
    buffer.update_cursor_mode(CursorMode::Normal).ok();
}

pub(super) fn insert_mode_line_end(buffer: &mut Buffer) {
    buffer.offset = buffer.len_bytes(buffer.index);

    if buffer.index < buffer.len_lines() - 1 {
        buffer.offset -= 1;
    }

    buffer.update_cursor_mode(CursorMode::Insert).ok();
}

pub(super) fn insert_mode_line_start(buffer: &mut Buffer) {
    buffer.offset = 0;
    buffer.update_cursor_mode(CursorMode::Insert).ok();
}

pub(super) fn insert_mode_line_next(buffer: &mut Buffer) {
    let index = buffer.index;
    let line_start_byte = buffer.text.line_to_byte(index + 1);
    buffer.text.insert_char(line_start_byte, '\n');

    super::move_cursor(buffer, super::movement::CursorMove::Down(1));

    buffer.offset = 0;
    buffer.update_cursor_mode(CursorMode::Insert).ok();
}

pub(super) fn insert_mode_line_prev(buffer: &mut Buffer) {
    let index = buffer.index;
    let line_start_byte = buffer.text.line_to_byte(index);
    buffer.text.insert_char(line_start_byte, '\n');

    buffer.offset = 0;
    buffer.update_cursor_mode(CursorMode::Insert).ok();
}
