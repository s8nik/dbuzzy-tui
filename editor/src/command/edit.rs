use crate::{buffer::Buffer, cursor};

pub(super) fn insert_char(buffer: &mut Buffer, ch: char) {
    let pos = buffer.position();
    buffer.text.insert_char(pos, ch);
    cursor!(buffer, offset += 1);
}

pub(super) fn new_line(buffer: &mut Buffer) {
    let pos = buffer.position();
    buffer.text.insert_char(pos, '\n');

    super::shift_cursor(buffer, super::shift::Shift::Down(1));
    cursor!(buffer, offset = 0);
}

pub(super) fn delete_char(buffer: &mut Buffer) {
    let pos = buffer.position();

    if pos < buffer.text.len_chars() {
        buffer.text.remove(pos..pos + 1);
    }
}

pub(super) fn backspace(buffer: &mut Buffer) {
    let pos = buffer.position();

    if pos > 0 {
        super::shift_cursor(buffer, super::shift::Shift::Left);
        buffer.text.remove(pos - 1..pos);
    }
}
