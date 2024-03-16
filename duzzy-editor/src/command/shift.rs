use crate::{
    buffer::{Buffer, Position},
    editor::Workspace,
    set_cursor,
};

enum Shift {
    Up(usize),
    Down(usize),
    Left,
    Right,
    Top,
    Bottom,
    LineStart,
    LineEnd,
}

pub(super) fn move_left(editor: &mut Workspace) {
    shift_cursor(editor, Shift::Left)
}

pub(super) fn move_down(editor: &mut Workspace) {
    shift_cursor(editor, Shift::Down(1))
}

pub(super) fn move_up(editor: &mut Workspace) {
    shift_cursor(editor, Shift::Up(1))
}

pub(super) fn move_right(editor: &mut Workspace) {
    shift_cursor(editor, Shift::Right)
}

pub(super) fn go_to_top_line(editor: &mut Workspace) {
    shift_cursor(editor, Shift::Top)
}

pub(super) fn go_to_bottom_line(editor: &mut Workspace) {
    shift_cursor(editor, Shift::Bottom)
}

pub(super) fn go_to_line_end(editor: &mut Workspace) {
    shift_cursor(editor, Shift::LineEnd)
}

pub(super) fn go_to_line_start(editor: &mut Workspace) {
    shift_cursor(editor, Shift::LineStart)
}

fn shift_cursor(editor: &mut Workspace, shift: Shift) {
    let buf = editor.curr_mut().buf_mut();
    let index = buf.pos.index;

    let pos = match shift {
        Shift::Up(n) => shift_up(n, buf),
        Shift::Down(n) => shift_down(n, buf),
        Shift::Left => shift_left(buf),
        Shift::Right => shift_right(buf),
        Shift::Top => (0, 0).into(),
        Shift::Bottom => (buf.len_lines() - 1, 0).into(),
        Shift::LineStart => (index, 0).into(),
        Shift::LineEnd => (index, buf.len_bytes(index).saturating_sub(1)).into(),
    };

    set_cursor!(buf, pos);
}

pub(super) fn shift_up(n: usize, buf: &mut Buffer) -> Position {
    let (index, offset) = Into::into(&buf.pos);

    let index = index.saturating_sub(n);
    let offset = offset.min(buf.len_bytes(index).saturating_sub(1));

    (index, offset).into()
}

pub(super) fn shift_down(n: usize, buf: &mut Buffer) -> Position {
    let (index, offset) = Into::into(&buf.pos);

    let index = (index + n).min(buf.len_lines() - 1);
    let offset = offset.min(buf.len_bytes(index).saturating_sub(1));

    (index, offset).into()
}

pub(super) fn shift_left(buf: &mut Buffer) -> Position {
    let (index, offset) = Into::into(&buf.pos);

    match (offset > 0, index > 0) {
        (true, _) => (index, offset - 1),
        (false, true) => (index - 1, buf.len_bytes(index - 1) - 1),
        _ => (index, offset),
    }
    .into()
}

pub(super) fn shift_right(buf: &mut Buffer) -> Position {
    let (index, offset) = Into::into(&buf.pos);

    match (
        offset < buf.len_bytes(index).saturating_sub(1),
        index < buf.len_lines().saturating_sub(1),
    ) {
        (true, _) => (index, offset + 1),
        (false, true) => ((index + 1).min(buf.len_lines() - 1), 0),
        (false, false) => (index, (offset + 1).min(buf.len_bytes(index))),
    }
    .into()
}
