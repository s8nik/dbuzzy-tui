use crate::{
    buffer::{Buffer, Pos},
    editor::Workspace,
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
    NextWordStart,
    NextWordEnd,
    PrevWordStart,
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

pub(super) fn next_word_start(ws: &mut Workspace) {
    shift_cursor(ws, Shift::NextWordStart);
}

pub(super) fn next_word_end(ws: &mut Workspace) {
    shift_cursor(ws, Shift::NextWordEnd);
}

pub(super) fn prev_word_start(ws: &mut Workspace) {
    shift_cursor(ws, Shift::PrevWordStart);
}

fn shift_cursor(ws: &mut Workspace, shift: Shift) {
    let buf = ws.curr_mut().buf_mut();
    let idx = buf.index();

    let pos = match shift {
        Shift::Up(n) => shift_up(n, buf),
        Shift::Down(n) => shift_down(n, buf),
        Shift::Left => shift_left(buf),
        Shift::Right => shift_right(buf),
        Shift::NextWordStart => shift_next_word_start(buf),
        Shift::NextWordEnd => shift_next_word_end(buf),
        Shift::PrevWordStart => shift_prev_word_start(buf),
        Shift::Top => (0, 0),
        Shift::Bottom => (buf.len_lines() - 1, 0),
        Shift::LineStart => (idx, 0),
        Shift::LineEnd => (idx, buf.len_bytes(idx).saturating_sub(1)),
    };

    buf.set_pos(pos);
    buf.update_selection(buf.byte_pos());
}

fn shift_next_word_start(buf: &mut Buffer) -> Pos {
    todo!()
}

fn shift_next_word_end(buf: &mut Buffer) -> Pos {
    todo!()
}

fn shift_prev_word_start(buf: &mut Buffer) -> Pos {
    todo!()
}

pub(super) fn shift_up(n: usize, buf: &mut Buffer) -> Pos {
    let (idx, ofs) = buf.pos();

    let idx = idx.saturating_sub(n);
    let ofs = ofs.min(buf.len_bytes(idx).saturating_sub(1));

    (idx, ofs)
}

pub(super) fn shift_down(n: usize, buf: &mut Buffer) -> Pos {
    let (idx, ofs) = buf.pos();

    let idx = (idx + n).min(buf.len_lines() - 1);
    let ofs = ofs.min(buf.len_bytes(idx).saturating_sub(1));

    (idx, ofs)
}

pub(super) fn shift_left(buf: &mut Buffer) -> Pos {
    let (idx, ofs) = buf.pos();

    match (ofs > 0, idx > 0) {
        (true, _) => (idx, ofs - 1),
        (false, true) => (idx - 1, buf.len_bytes(idx - 1).saturating_sub(1)),
        _ => (idx, ofs),
    }
}

pub(super) fn shift_right(buf: &mut Buffer) -> Pos {
    let (idx, ofs) = buf.pos();

    match (
        ofs < buf.len_bytes(idx).saturating_sub(1),
        idx < buf.len_lines().saturating_sub(1),
    ) {
        (true, _) => (idx, ofs + 1),
        (false, true) => ((idx + 1).min(buf.len_lines() - 1), 0),
        (false, false) => (idx, (ofs + 1).min(buf.len_bytes(idx))),
    }
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
            buf.text_mut().insert(0, "test\n\ntest");
        }

        shift_cursor(&mut ws, Shift::Up(10));
        assert_eq!((0, 0), ws.curr().buf().pos());

        shift_cursor(&mut ws, Shift::Bottom);
        assert_eq!((2, 0), ws.curr().buf().pos());

        shift_cursor(&mut ws, Shift::Top);
        assert_eq!((0, 0), ws.curr().buf().pos());

        shift_cursor(&mut ws, Shift::Right);
        assert_eq!((0, 1), ws.curr().buf().pos());

        shift_cursor(&mut ws, Shift::Down(1));
        assert_eq!((1, 0), ws.curr().buf().pos());

        shift_cursor(&mut ws, Shift::LineEnd);
        assert_eq!((1, 0), ws.curr().buf().pos());

        shift_cursor(&mut ws, Shift::Left);
        assert_eq!((0, 4), ws.curr().buf().pos());

        shift_cursor(&mut ws, Shift::Right);
        assert_eq!((1, 0), ws.curr().buf().pos());

        shift_cursor(&mut ws, Shift::Right);
        assert_eq!((2, 0), ws.curr().buf().pos());

        shift_cursor(&mut ws, Shift::LineEnd);
        assert_eq!((2, 3), ws.curr().buf().pos());

        shift_cursor(&mut ws, Shift::Right);
        assert_eq!((2, 4), ws.curr().buf().pos());
    }
}
