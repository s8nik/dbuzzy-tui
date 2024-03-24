use crate::{
    buffer::{Buffer, CursorMode},
    document::Document,
    editor::Workspace,
    set_cursor,
    transaction::TransactionResult,
};

enum Switch {
    Inplace,
    LineStart,
    LineEnd,
    LineNext,
    LinePrev,
}

pub(super) fn normal_mode_inplace(ws: &mut Workspace) {
    let doc = ws.curr_mut();
    doc.with_transaction(|_, buf| {
        buf.mode = CursorMode::Normal;
        TransactionResult::Commit
    });
}

pub(super) fn insert_mode_inplace(ws: &mut Workspace) {
    switch_mode(ws, Switch::Inplace);
}

pub(super) fn insert_mode_line_end(ws: &mut Workspace) {
    switch_mode(ws, Switch::LineEnd);
}

pub(super) fn insert_mode_line_start(ws: &mut Workspace) {
    switch_mode(ws, Switch::LineStart);
}

pub(super) fn insert_mode_line_next(ws: &mut Workspace) {
    switch_mode(ws, Switch::LineNext);
}

pub(super) fn insert_mode_line_prev(ws: &mut Workspace) {
    switch_mode(ws, Switch::LinePrev);
}

fn switch_mode(ws: &mut Workspace, switch: Switch) {
    let doc = ws.curr_mut();

    match switch {
        Switch::LineStart => set_cursor!(doc.buf_mut(), offset = 0),
        Switch::LineEnd => switch_line_end(doc.buf_mut()),
        Switch::LineNext => switch_line_next(doc),
        Switch::LinePrev => switch_line_prev(doc),
        _ => (),
    };

    doc.buf_mut().mode = CursorMode::Insert;
}

fn switch_line_end(buf: &mut Buffer) {
    let index = buf.pos.index;
    set_cursor!(buf, offset = buf.len_bytes(index));

    if index < buf.len_lines() - 1 {
        set_cursor!(buf, offset -= 1);
    }
}

fn switch_line_next(doc: &mut Document) {
    let buf = doc.buf();
    let start_pos = buf.text.line_to_byte(buf.pos.index + 1);

    switch_with_new_line(doc, start_pos);

    let buf = doc.buf_mut();
    set_cursor!(buf, super::shift_down(1, buf));
}

fn switch_line_prev(doc: &mut Document) {
    let buf = doc.buf();
    let start_pos = buf.text.line_to_byte(buf.pos.index);

    switch_with_new_line(doc, start_pos);
    set_cursor!(doc.buf_mut(), offset = 0);
}

fn switch_with_new_line(doc: &mut Document, start_pos: usize) {
    doc.with_transaction(|tx, buf| {
        tx.shift(buf.byte_pos());
        tx.insert_char(start_pos, '\n');
        tx.shift(start_pos);
        tx.apply(&mut buf.text);

        TransactionResult::Keep
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switch_next() {
        let mut doc = Document::default();

        switch_line_next(&mut doc);

        doc.commit();

        switch_line_next(&mut doc);

        let buf = doc.buf();
        assert_eq!((2, 0), Into::into(&buf.pos));
        assert_eq!(&buf.text.to_string(), "\n\n");
    }

    #[test]
    fn test_switch_prev() {
        let mut doc = Document::default();

        switch_line_prev(&mut doc);

        doc.commit();

        switch_line_prev(&mut doc);

        let buf = doc.buf();
        assert_eq!((0, 0), Into::into(&buf.pos));
        assert_eq!(&buf.text.to_string(), "\n\n");
    }
}
