use crate::{buffer::Buffer, cursor, workspace::Document};

pub(super) fn insert_char(doc: &mut Document, ch: char) {
    let mut buf = doc.buf_mut();
    let pos = buf.position();
    buf.text.insert_char(pos, ch);
    cursor!(buf, offset += 1);
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
