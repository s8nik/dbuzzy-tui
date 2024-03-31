use crate::{
    buffer::{Buffer, Mode},
    document::Document,
    editor::Workspace,
    transaction::TransactionResult,
};

enum Switch {
    Inplace,
    LineStart,
    LineEnd,
    LineNext,
    LinePrev,
}

pub(super) fn normal_mode(ws: &mut Workspace) {
    let doc = ws.curr_mut();

    match doc.buf().mode() {
        Mode::Visual => doc.buf_mut().set_mode(Mode::Normal),
        Mode::Insert => doc.with_transaction(|_, buf| {
            buf.set_mode(Mode::Normal);
            TransactionResult::Commit
        }),
        _ => (),
    }
}

pub(super) fn visual_mode(ws: &mut Workspace) {
    let buf = ws.curr_mut().buf_mut();
    let pos = buf.char_pos();

    buf.new_selection(pos);
    buf.set_mode(Mode::Visual);
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
        Switch::LineStart => doc.buf_mut().set_offset(0),
        Switch::LineEnd => switch_line_end(doc.buf_mut()),
        Switch::LineNext => switch_line_next(doc),
        Switch::LinePrev => switch_line_prev(doc),
        _ => (),
    };

    doc.buf_mut().set_mode(Mode::Insert);
}

fn switch_line_end(buf: &mut Buffer) {
    let idx = buf.index();
    buf.set_offset(buf.len_bytes(idx));

    if idx < buf.len_lines() - 1 {
        let ofs = buf.offset() - 1;
        buf.set_offset(ofs);
    }
}

fn switch_line_next(doc: &mut Document) {
    let buf = doc.buf();
    let idx = buf.index() + 1;
    let line_pos = buf.line_byte(idx);

    switch_with_new_line(doc, line_pos);

    let buf = doc.buf_mut();
    let new_pos = super::shift_down(1, buf);
    buf.set_pos(new_pos);
}

fn switch_line_prev(doc: &mut Document) {
    let buf = doc.buf();
    let idx = buf.index() + 1;
    let line_pos = buf.line_byte(idx);

    doc.buf_mut().set_offset(0);
    switch_with_new_line(doc, line_pos);
}

fn switch_with_new_line(doc: &mut Document, line_pos: usize) {
    doc.with_transaction(|tx, buf| {
        tx.shift(buf.byte_pos());
        tx.insert_char(line_pos, '\n');
        tx.shift(line_pos);
        tx.apply(buf.text_mut());

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
        assert_eq!((2, 0), buf.pos());
        assert_eq!(&buf.text().to_string(), "\n\n");
    }

    #[test]
    fn test_switch_prev() {
        let mut doc = Document::default();

        switch_line_prev(&mut doc);

        doc.commit();

        switch_line_prev(&mut doc);

        let buf = doc.buf();
        assert_eq!((0, 0), buf.pos());
        assert_eq!(&buf.text().to_string(), "\n\n");
    }
}
