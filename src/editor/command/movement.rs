use crate::editor::{buffer::Content, cursor::CursorMode};

pub(super) fn move_forward(Content { text, cursor }: &mut Content) {
    if cursor.offset < text.line(cursor.index).len_bytes() {
        cursor.offset += 1;
    } else if cursor.index < text.len_lines() - 1 {
        cursor.index += 1;
        cursor.offset = 0;
    }
}

pub(super) fn move_back(Content { text, cursor }: &mut Content) {
    if cursor.offset > 0 {
        cursor.offset -= 1;
    } else if cursor.index > 0 {
        cursor.index -= 1;
        cursor.offset = text.line(cursor.index).len_bytes() - 1;
    }
}

pub(super) fn move_up(Content { text, cursor }: &mut Content) {
    if cursor.index > 0 {
        cursor.index -= 1;
        cursor.offset = cursor.offset.min(text.line(cursor.index).len_bytes());
    }
}

pub(super) fn move_down(Content { text, cursor }: &mut Content) {
    if cursor.index < text.len_lines() - 1 {
        cursor.index += 1;
        cursor.offset = cursor.offset.min(text.line(cursor.index).len_bytes());
    }
}

pub(super) fn insert_mode_line_end(Content { text, cursor }: &mut Content) {
    cursor.offset = text.line(cursor.index).len_bytes();

    if cursor.index < text.len_lines() - 1 {
        cursor.offset -= 1;
    }

    cursor.mode = CursorMode::Insert;
}

pub(super) fn insert_mode_line_start(content: &mut Content) {
    content.cursor.offset = 0;
    content.cursor.mode = CursorMode::Insert;
}

pub(super) fn go_to_start_line(Content { cursor, .. }: &mut Content) {
    cursor.index = 0;
    cursor.offset = 0;
}

pub(super) fn go_to_end_line(Content { text, cursor }: &mut Content) {
    cursor.index = text.len_lines() - 1;
    cursor.offset = 0;
}

pub(super) fn go_to_start_curr_line(content: &mut Content) {
    content.cursor.offset = 0;
}

pub(super) fn go_to_end_curr_line(Content { text, cursor }: &mut Content) {
    cursor.offset = text.line(cursor.index).len_bytes();
}
