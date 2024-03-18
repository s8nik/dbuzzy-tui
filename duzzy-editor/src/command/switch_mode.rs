use crate::{
    buffer::{Buffer, CursorMode},
    doc_mut,
    editor::Workspace,
    set_cursor,
};

enum Switch {
    Inplace,
    LineStart,
    LineEnd,
    LineNext,
    LinePrev,
}

pub(super) fn normal_mode_inplace(ws: &mut Workspace) {
    let (buf, history) = doc_mut!(ws);

    buf.mode = CursorMode::Normal;
    history.commit();
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
    let (buf, history) = doc_mut!(ws);

    match switch {
        Switch::LineStart => set_cursor!(buf, offset = 0),
        Switch::LineEnd => switch_line_end(buf),
        Switch::LineNext => switch_line_next(buf),
        Switch::LinePrev => switch_line_prev(buf),
        _ => (),
    };

    buf.mode = CursorMode::Insert;
    history.commit();
}

fn switch_line_end(buf: &mut Buffer) {
    let index = buf.pos.index;
    set_cursor!(buf, offset = buf.len_bytes(index));

    if index < buf.len_lines() - 1 {
        set_cursor!(buf, offset -= 1);
    }
}

fn switch_line_next(buf: &mut Buffer) {
    let index = buf.pos.index;

    let line_start_byte = buf.text.line_to_byte(index + 1);
    buf.text.insert_char(line_start_byte, '\n');
    set_cursor!(buf, super::shift_down(1, buf));
}

fn switch_line_prev(buf: &mut Buffer) {
    let index = buf.pos.index;

    let line_start_byte = buf.text.line_to_byte(index);
    buf.text.insert_char(line_start_byte, '\n');
    set_cursor!(buf, offset = 0);
}
