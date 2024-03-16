use crate::{
    buffer::{Buffer, CursorMode},
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

pub(super) fn insert_mode_inplace(editor: &mut Workspace) {
    switch_mode(editor, Switch::Inplace);
}

pub(super) fn insert_mode_line_end(editor: &mut Workspace) {
    switch_mode(editor, Switch::LineEnd);
}

pub(super) fn insert_mode_line_start(editor: &mut Workspace) {
    switch_mode(editor, Switch::LineStart);
}

pub(super) fn insert_mode_line_next(editor: &mut Workspace) {
    switch_mode(editor, Switch::LineNext);
}

pub(super) fn insert_mode_line_prev(editor: &mut Workspace) {
    switch_mode(editor, Switch::LinePrev);
}

fn switch_mode(editor: &mut Workspace, switch: Switch) {
    let buf = editor.curr_mut().buf_mut();

    match switch {
        Switch::LineStart => set_cursor!(buf, offset = 0),
        Switch::LineEnd => switch_line_end(buf),
        Switch::LineNext => switch_line_next(buf),
        Switch::LinePrev => switch_line_prev(buf),
        _ => (),
    };

    buf.mode = CursorMode::Insert;
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
