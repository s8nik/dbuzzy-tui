use crate::editor::Workspace;

pub(super) fn undo(ws: &mut Workspace) {
    let doc = ws.curr_mut();

    if let Some(pos) = doc.undo() {
        let buf = doc.buf_mut();
        buf.set_pos(buf.curs_pos(pos));
    }
}

pub(super) fn redo(ws: &mut Workspace) {
    let doc = ws.curr_mut();

    if let Some(pos) = doc.redo() {
        let buf = doc.buf_mut();
        buf.set_pos(buf.curs_pos(pos));
    }
}
