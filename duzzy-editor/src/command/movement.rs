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

pub(super) fn move_left(ws: &mut Workspace) {
    shift_cursor(ws, Shift::Left);
}

pub(super) fn move_down(ws: &mut Workspace) {
    shift_cursor(ws, Shift::Down(1));
}

pub(super) fn move_up(ws: &mut Workspace) {
    shift_cursor(ws, Shift::Up(1));
}

pub(super) fn move_right(ws: &mut Workspace) {
    shift_cursor(ws, Shift::Right);
}

pub(super) fn go_to_top_line(ws: &mut Workspace) {
    shift_cursor(ws, Shift::Top);
}

pub(super) fn go_to_bottom_line(ws: &mut Workspace) {
    shift_cursor(ws, Shift::Bottom);
}

pub(super) fn go_to_line_end(ws: &mut Workspace) {
    shift_cursor(ws, Shift::LineEnd);
}

pub(super) fn go_to_line_start(ws: &mut Workspace) {
    shift_cursor(ws, Shift::LineStart);
}

fn shift_cursor(ws: &mut Workspace, shift: Shift) {
    let buf = ws.curr_mut().buf_mut();
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
        (false, true) => (index - 1, buf.len_bytes(index - 1).saturating_sub(1)),
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

#[cfg(test)]
mod tests {
    use crate::document::Document;

    use super::*;

    #[test]
    fn test_movement() {
        let mut ws = Workspace::default();
        ws.add_doc(Document::default());

        {
            let buf = ws.curr_mut().buf_mut();
            buf.text = ropey::Rope::from_str("test\n\ntest");
        }

        shift_cursor(&mut ws, Shift::Up(10));
        let buf = ws.curr().buf();
        assert_eq!((0, 0), Into::into(&buf.pos));

        shift_cursor(&mut ws, Shift::Bottom);
        let buf = ws.curr().buf();
        assert_eq!((2, 0), Into::into(&buf.pos));

        shift_cursor(&mut ws, Shift::Top);
        let buf = ws.curr().buf();
        assert_eq!((0, 0), Into::into(&buf.pos));

        shift_cursor(&mut ws, Shift::Right);
        let buf = ws.curr().buf();
        assert_eq!((0, 1), Into::into(&buf.pos));

        shift_cursor(&mut ws, Shift::Down(1));
        let buf = ws.curr().buf();
        assert_eq!((1, 0), Into::into(&buf.pos));

        shift_cursor(&mut ws, Shift::LineEnd);
        let buf = ws.curr().buf();
        assert_eq!((1, 0), Into::into(&buf.pos));

        shift_cursor(&mut ws, Shift::Left);
        let buf = ws.curr().buf();
        assert_eq!((0, 4), Into::into(&buf.pos));

        shift_cursor(&mut ws, Shift::Right);
        let buf = ws.curr().buf();
        assert_eq!((1, 0), Into::into(&buf.pos));

        shift_cursor(&mut ws, Shift::Right);
        let buf = ws.curr().buf();
        assert_eq!((2, 0), Into::into(&buf.pos));

        shift_cursor(&mut ws, Shift::LineEnd);
        let buf = ws.curr().buf();
        assert_eq!((2, 3), Into::into(&buf.pos));

        shift_cursor(&mut ws, Shift::Right);
        let buf = ws.curr().buf();
        assert_eq!((2, 4), Into::into(&buf.pos));
    }
}
