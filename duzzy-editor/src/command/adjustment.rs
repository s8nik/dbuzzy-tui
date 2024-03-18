use crate::{doc_mut, editor::Workspace, set_cursor};

pub(super) fn insert_char(ws: &mut Workspace, ch: char) {
    let (buf, history) = doc_mut!(ws);

    let pos = buf.pos;
    let text_pos = buf.text_pos();
    buf.text.insert_char(text_pos, ch);

    set_cursor!(buf, offset += 1);
    super::history::insert_char(ch, history, pos, buf.pos);
}

pub(super) fn new_line(ws: &mut Workspace) {
    let (buf, history) = doc_mut!(ws);

    let pos = buf.pos;
    let text_pos = buf.text_pos();
    buf.text.insert_char(text_pos, '\n');

    set_cursor!(buf, super::shift_down(1, buf));
    set_cursor!(buf, offset = 0);
    super::history::insert_char('\n', history, pos, buf.pos);
}

pub(super) fn delete_char_inplace(ws: &mut Workspace) {
    let (buf, history) = doc_mut!(ws);

    let text_pos = buf.text_pos();

    if text_pos < buf.text.len_chars() {
        let slice = buf.text.slice(text_pos..text_pos + 1).to_string();
        buf.text.remove(text_pos..text_pos + 1);
        super::history::delete_slice(slice.into(), history, buf.pos, buf.pos);
        history.commit();
    }
}

pub(super) fn delete_char(ws: &mut Workspace) {
    let (buf, history) = doc_mut!(ws);

    let pos = buf.pos;
    let text_pos = buf.text_pos();

    if text_pos > 0 {
        set_cursor!(buf, super::shift_left(buf));
        let slice = buf.text.slice(text_pos - 1..text_pos).to_string();
        buf.text.remove(text_pos - 1..text_pos);
        super::history::delete_slice(slice.into(), history, pos, buf.pos);
    }
}
