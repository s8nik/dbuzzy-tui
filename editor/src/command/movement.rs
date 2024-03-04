use crate::buffer::{Buffer, CursorMode};

pub(super) fn move_forward(buffer: &mut Buffer) {
    let offset = buffer.offset();
    let index = buffer.index();

    if offset < buffer.line_len_bytes() {
        buffer.update_offset(offset + 1);
    } else if index < buffer.len_lines() - 1 {
        buffer.update_offset(0);
        buffer.update_index(index + 1);
    }
}

pub(super) fn move_back(buffer: &mut Buffer) {
    let offset = buffer.offset();
    let index = buffer.index();

    if offset > 0 {
        buffer.update_offset(offset - 1);
    } else if index > 0 {
        buffer.update_index(index - 1);
        buffer.update_offset(buffer.line_len_bytes() - 1);
    }
}

pub(super) fn move_up(buffer: &mut Buffer) {
    let offset = buffer.offset();
    let index = buffer.index();

    if index > 0 {
        buffer.update_index(index - 1);
        buffer.update_offset(offset.min(buffer.line_len_bytes()))
    }
}

pub(super) fn move_down(buffer: &mut Buffer) {
    let offset = buffer.offset();
    let index = buffer.index();

    if index < buffer.len_lines() - 1 {
        buffer.update_index(index + 1);
        buffer.update_offset(offset.min(buffer.line_len_bytes()))
    }
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

pub(super) fn go_to_start_line(buffer: &mut Buffer) {
    buffer.update_index(0);
    buffer.update_offset(0);
}

pub(super) fn go_to_end_line(buffer: &mut Buffer) {
    buffer.update_index(buffer.len_lines() - 1);
    buffer.update_offset(0);
}

pub(super) fn go_to_start_curr_line(buffer: &mut Buffer) {
    buffer.update_offset(0)
}

pub(super) fn go_to_end_curr_line(buffer: &mut Buffer) {
    buffer.update_offset(buffer.line_len_bytes());
}
