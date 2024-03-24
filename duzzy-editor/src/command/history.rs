use crate::{editor::Workspace, set_cursor};

pub(super) fn undo(ws: &mut Workspace) {
    let doc = ws.curr_mut();

    if let Some(pos) = doc.undo() {
        let buf = doc.buf_mut();
        set_cursor!(buf, buf.cursor_pos(pos));
    }
}

pub(super) fn redo(ws: &mut Workspace) {
    let doc = ws.curr_mut();

    if let Some(pos) = doc.redo() {
        let buf = doc.buf_mut();
        set_cursor!(buf, buf.cursor_pos(pos));
    }
}
