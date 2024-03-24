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
    let buf = doc.buf_mut();

    let index = buf.pos.index;
    let new_line = '\n';

    let line_start_byte = buf.text.line_to_byte(index + 1);
    buf.text.insert_char(line_start_byte, new_line);

    set_cursor!(buf, super::shift_down(1, buf));
}

fn switch_line_prev(doc: &mut Document) {
    let buf = doc.buf_mut();

    let index = buf.pos.index;
    let new_line = '\n';

    let line_start_byte = buf.text.line_to_byte(index);
    buf.text.insert_char(line_start_byte, new_line);

    set_cursor!(buf, offset = 0);
}
