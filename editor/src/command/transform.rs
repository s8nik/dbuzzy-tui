use crate::buffer::Buffer;

pub(super) fn insert_char(buffer: &mut Buffer, ch: char) {
    let pos = buffer.position();
    buffer.text_mut().insert_char(pos, ch);

    super::move_forward(buffer);
}

pub(super) fn new_line(buffer: &mut Buffer) {
    let pos = buffer.position();
    buffer.text_mut().insert_char(pos, '\n');

    super::move_down(buffer);
    buffer.update_offset(0)
}

pub(super) fn delete_char(buffer: &mut Buffer) {
    let pos = buffer.position();

    if pos < buffer.text().len_chars() {
        buffer.text_mut().remove(pos..pos + 1);
    }
}

pub(super) fn delete_char_backspace(buffer: &mut Buffer) {
    let pos = buffer.position();

    if pos > 0 {
        super::move_back(buffer);
        buffer.text_mut().remove(pos - 1..pos);
    }
}
