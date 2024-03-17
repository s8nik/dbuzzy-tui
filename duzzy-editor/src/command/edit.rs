use crate::{editor::Workspace, set_cursor};

pub(super) fn insert_char(workspace: &mut Workspace, ch: char) {
    let buf = workspace.curr_mut().buf_mut();

    let pos = buf.text_pos();
    buf.text.insert_char(pos, ch);

    set_cursor!(buf, offset += 1);
}

pub(super) fn new_line(workspace: &mut Workspace) {
    let buf = workspace.curr_mut().buf_mut();

    let pos = buf.text_pos();
    buf.text.insert_char(pos, '\n');

    set_cursor!(buf, super::shift_down(1, buf));
    set_cursor!(buf, offset = 0);
}

pub(super) fn delete_char_inplace(workspace: &mut Workspace) {
    let buf = workspace.curr_mut().buf_mut();
    let pos = buf.text_pos();

    if pos < buf.text.len_chars() {
        buf.text.remove(pos..pos + 1);
    }
}

pub(super) fn delete_char(workspace: &mut Workspace) {
    let buf = workspace.curr_mut().buf_mut();
    let pos = buf.text_pos();

    if pos > 0 {
        set_cursor!(buf, super::shift_left(buf));
        buf.text.remove(pos - 1..pos);
    }
}
