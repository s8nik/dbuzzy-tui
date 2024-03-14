use crate::{
    buffer::{Buffer, CursorMode},
    cursor,
};

pub enum Switch {
    Inplace,
    LineStart,
    LineEnd,
    LineNext,
    LinePrev,
}

pub(super) fn switch_mode(buffer: &mut Buffer, switch: Switch) {
    match switch {
        Switch::LineStart => cursor!(buffer, offset 0),
        Switch::LineEnd => switch_line_end(buffer),
        Switch::LineNext => switch_line_next(buffer),
        Switch::LinePrev => switch_line_prev(buffer),
        _ => (),
    };

    buffer.update_cursor_mode(CursorMode::Insert);
}

fn switch_line_end(buffer: &mut Buffer) {
    let (index, _) = cursor!(buffer);
    cursor!(buffer, offset buffer.len_bytes(index));

    if index < buffer.len_lines() - 1 {
        cursor!(buffer, offset -= 1);
    }
}

fn switch_line_next(buffer: &mut Buffer) {
    let line_start_byte = buffer.text.line_to_byte(buffer.index + 1);
    buffer.text.insert_char(line_start_byte, '\n');
    super::shift_cursor(buffer, super::shift::Shift::Down(1));
}

fn switch_line_prev(buffer: &mut Buffer) {
    let line_start_byte = buffer.text.line_to_byte(buffer.index);
    buffer.text.insert_char(line_start_byte, '\n');
    cursor!(buffer, offset 0);
}
