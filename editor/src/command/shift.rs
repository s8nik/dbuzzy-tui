use crate::buffer::Buffer;

pub enum Shift {
    Up(usize),
    Down(usize),
    Left,
    Right,
    Top,
    Bottom,
    LineStart,
    LineEnd,
}

pub(super) fn shift_cursor(buffer: &mut Buffer, direction: Shift) {
    let offset = buffer.offset;
    let index = buffer.index;

    let (new_offset, new_index) = match direction {
        Shift::Up(n) => {
            let index = index.saturating_sub(n);
            let offset = offset.min(buffer.len_bytes(index).saturating_sub(1));
            (offset, index)
        }
        Shift::Down(n) => {
            let index = (index + n).min(buffer.len_lines() - 1);
            let offset = offset.min(buffer.len_bytes(index).saturating_sub(1));
            (offset, index)
        }
        Shift::Left => match (offset > 0, index > 0) {
            (true, _) => (offset - 1, index),
            (false, true) => (buffer.len_bytes(index - 1) - 1, index - 1),
            _ => (offset, index),
        },
        Shift::Right => match (
            offset < buffer.len_bytes(index).saturating_sub(1),
            index < buffer.len_lines().saturating_sub(1),
        ) {
            (true, _) => (offset + 1, index),
            (false, true) => (0, (index + 1).min(buffer.len_lines() - 1)),
            (false, false) => ((offset + 1).min(buffer.len_bytes(index)), index),
        },
        Shift::Top => (0, 0),
        Shift::Bottom => (0, buffer.len_lines() - 1),
        Shift::LineStart => (0, index),
        Shift::LineEnd => (buffer.len_bytes(index).saturating_sub(1), index),
    };

    buffer.offset = new_offset;
    buffer.index = new_index;
}
