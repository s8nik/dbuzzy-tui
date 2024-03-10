use crate::buffer::{Buffer, CursorMode};

pub enum Switch {
    Inplace,
    LineStart,
    LineEnd,
    LineNext,
    LinePrev,
}

pub(super) fn switch_mode(buffer: &mut Buffer, switch: Switch) {
    match switch {
        Switch::LineStart => buffer.offset = 0,
        Switch::LineEnd => switch_line_end(buffer),
        Switch::LineNext => switch_line_next(buffer),
        Switch::LinePrev => switch_line_prev(buffer),
        _ => (),
    };

    buffer.update_cursor_mode(CursorMode::Insert)
}

fn switch_line_end(buffer: &mut Buffer) {
    let index = buffer.index;
    buffer.offset = buffer.len_bytes(index);

    if index < buffer.len_lines() - 1 {
        buffer.offset -= 1;
    }
}

fn switch_line_next(buffer: &mut Buffer) {
    let line_start_byte = buffer.text.line_to_byte(buffer.index + 1);
    buffer.text.insert_char(line_start_byte, '\n');
    super::move_cursor(buffer, super::movement::CursorMove::Down(1));
}

fn switch_line_prev(buffer: &mut Buffer) {
    let line_start_byte = buffer.text.line_to_byte(buffer.index);
    buffer.text.insert_char(line_start_byte, '\n');
    buffer.offset = 0;
}
