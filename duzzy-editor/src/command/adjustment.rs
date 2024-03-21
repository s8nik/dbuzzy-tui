use crate::{doc_mut, editor::Workspace, set_cursor};

pub(super) fn insert_char(ws: &mut Workspace, ch: char) {
    let (buf, history) = doc_mut!(ws);

    let pos = buf.byte_pos();
    buf.text.insert_char(pos, ch);

    super::history::insert_char(ch, pos, history);

    set_cursor!(buf, offset += 1);
}

pub(super) fn new_line(ws: &mut Workspace) {
    let (buf, history) = doc_mut!(ws);
    let new_line = '\n';

    let pos = buf.byte_pos();
    buf.text.insert_char(pos, new_line);

    super::history::insert_char(new_line, pos, history);

    set_cursor!(buf, super::shift_down(1, buf));
    set_cursor!(buf, offset = 0);
}

pub(super) fn delete_char_inplace(ws: &mut Workspace) {
    let (buf, history) = doc_mut!(ws);

    let pos = buf.byte_pos();

    if pos < buf.text.len_chars() {
        let ch = buf.text.char(pos);
        buf.text.remove(pos..pos + 1);

        super::history::delete_char_inplace(ch, pos, history);
    }
}

pub(super) fn delete_char(ws: &mut Workspace) {
    let (buf, history) = doc_mut!(ws);

    let pos = buf.byte_pos();

    if pos > 0 {
        set_cursor!(buf, super::shift_left(buf));
        let ch = buf.text.char(pos - 1);
        buf.text.remove(pos - 1..pos);

        super::history::delete_char(ch, pos, history);
    }
}
