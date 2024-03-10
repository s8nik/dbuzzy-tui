use crate::buffer::Buffer;

pub enum CursorMove {
    Up(usize),
    Down(usize),
    Back,
    Forward,
    Top,
    Bottom,
    LineStart,
    LineEnd,
}

pub(super) fn move_cursor(buffer: &mut Buffer, direction: CursorMove) {
    let offset = buffer.offset;
    let index = buffer.index;

    let len_offset = if buffer.is_insert() { 0 } else { 1 };

    let (new_offset, new_index) = match direction {
        CursorMove::Up(n) => {
            let index = index.saturating_sub(n);
            let offset = offset.min(buffer.len_bytes(index) - 1);
            (offset, index)
        }
        CursorMove::Down(n) => {
            let index = (index + n).min(buffer.len_lines() - 1);
            let offset = offset.min(buffer.len_bytes(index) - 1);
            (offset, index)
        }
        CursorMove::Back => match (offset > 0, index > 0) {
            (true, _) => (offset - 1, index),
            (false, true) => (buffer.len_bytes(index - 1) - 1, index - 1),
            _ => (offset, index),
        },
        CursorMove::Forward => match (
            offset < buffer.len_bytes(index) - len_offset,
            index < buffer.len_lines() - 1,
        ) {
            (true, _) => (offset + 1, index),
            (false, true) => (0, (index + 1).min(buffer.len_lines() - 1)),
            _ => (offset, index),
        },
        CursorMove::Top => (0, 0),
        CursorMove::Bottom => (0, buffer.len_lines() - 1),
        CursorMove::LineStart => (0, index),
        CursorMove::LineEnd => (buffer.len_bytes(index) - 1, index),
    };

    buffer.offset = new_offset;
    buffer.index = new_index;
}
