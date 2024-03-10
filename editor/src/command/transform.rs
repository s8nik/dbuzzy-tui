use crate::buffer::Buffer;

pub(super) fn insert_char(buffer: &mut Buffer, ch: char) {
    let pos = buffer.position();
    buffer.text.insert_char(pos, ch);

    super::move_cursor(buffer, super::movement::CursorMove::Forward);
}

pub(super) fn new_line(buffer: &mut Buffer) {
    let pos = buffer.position();
    buffer.text.insert_char(pos, '\n');

    super::move_cursor(buffer, super::movement::CursorMove::Down(1));
    buffer.offset = 0;
}

pub(super) fn delete_char(buffer: &mut Buffer) {
    let pos = buffer.position();

    if pos < buffer.text.len_chars() {
        buffer.text.remove(pos..pos + 1);
    }
}

pub(super) fn delete_char_backspace(buffer: &mut Buffer) {
    let pos = buffer.position();

    if pos > 0 {
        super::move_cursor(buffer, super::movement::CursorMove::Back);
        buffer.text.remove(pos - 1..pos);
    }
}
