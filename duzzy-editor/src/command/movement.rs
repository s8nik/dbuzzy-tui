use crate::{
    buffer::{Buffer, Pos},
    editor::Workspace,
};

#[derive(PartialEq, Eq)]
enum Shift {
    Up(usize),
    Down(usize),
    Left,
    Right,
    Top,
    Bottom,
    LineStart,
    LineEnd,
    ByWord(ShiftWord),
}

#[derive(PartialEq, Eq)]
enum ShiftWord {
    NextStart,
    PrevStart,
    NextEnd,
}

pub(super) fn move_left(ws: &mut Workspace) {
    shift_cursor_impl(ws, Shift::Left);
}

pub(super) fn move_down(ws: &mut Workspace) {
    shift_cursor_impl(ws, Shift::Down(1));
}

pub(super) fn move_up(ws: &mut Workspace) {
    shift_cursor_impl(ws, Shift::Up(1));
}

pub(super) fn move_right(ws: &mut Workspace) {
    shift_cursor_impl(ws, Shift::Right);
}

pub(super) fn go_to_top_line(ws: &mut Workspace) {
    shift_cursor_impl(ws, Shift::Top);
}

pub(super) fn go_to_bottom_line(ws: &mut Workspace) {
    shift_cursor_impl(ws, Shift::Bottom);
}

pub(super) fn go_to_line_end(ws: &mut Workspace) {
    shift_cursor_impl(ws, Shift::LineEnd);
}

pub(super) fn go_to_line_start(ws: &mut Workspace) {
    shift_cursor_impl(ws, Shift::LineStart);
}

pub(super) fn move_next_word_end(ws: &mut Workspace) {
    shift_cursor_impl(ws, Shift::ByWord(ShiftWord::NextEnd));
}

pub(super) fn move_next_word_start(ws: &mut Workspace) {
    shift_cursor_impl(ws, Shift::ByWord(ShiftWord::NextStart));
}

pub(super) fn move_prev_word_start(ws: &mut Workspace) {
    shift_cursor_impl(ws, Shift::ByWord(ShiftWord::PrevStart));
}

fn shift_cursor_impl(ws: &mut Workspace, shift: Shift) {
    let buf = ws.curr_mut().buf_mut();
    let idx = buf.index();

    if !buf.is_visual() {
        buf.reset_selection();
    }

    let pos = match shift {
        Shift::Up(n) => shift_up(n, buf),
        Shift::Down(n) => shift_down(n, buf),
        Shift::Left => shift_left(buf),
        Shift::Right => shift_right(buf),
        Shift::Top => (0, 0),
        Shift::Bottom => (buf.len_lines() - 1, 0),
        Shift::LineStart => (idx, 0),
        Shift::LineEnd => (idx, buf.len_bytes(idx).saturating_sub(1)),
        Shift::ByWord(kind) => shift_by_word(buf, kind),
    };

    buf.set_pos(pos);
    if buf.selection().is_some() {
        buf.update_selection(buf.byte_pos());
    }
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

fn shift_by_word(buf: &mut Buffer, kind: ShiftWord) -> Pos {
    let (idx, ofs) = buf.pos();

    if buf.is_normal() {
        buf.new_selection(buf.byte_pos());
    }

    let len_lines = buf.len_lines();
    let line = buf.text().line(idx);

    // let slice = if kind == ShiftWord::PrevStart {
    //     line.slice(..ofs)
    // } else {
    //     line.slice(ofs..)
    // };

    // let slice = slice.to_string();
    let slice = line.to_string();

    let pos = match kind {
        ShiftWord::NextStart => shift_by_word_impl(slice.chars().enumerate(), |_, cur| {
            (cur.kind != CharKind::Space).then_some(cur.pos)
        }),
        ShiftWord::NextEnd => shift_by_word_impl(slice.chars().enumerate(), |prev, cur| {
            (prev.kind != CharKind::Space).then_some(cur.pos)
        }),
        ShiftWord::PrevStart => shift_by_word_impl(slice.chars().rev().enumerate(), |cur, next| {
            (cur.kind != CharKind::Space).then_some(ofs - next.pos)
        }),
    };

    let ofs = pos.unwrap_or(ofs);

    (idx, ofs).into()
}

fn shift_by_word_impl<F: Fn(Char, Char) -> Option<usize>>(
    mut it: impl Iterator<Item = (usize, char)>,
    is_not_space: F,
) -> Option<usize> {
    let mut prev = it.next()?.into();

    for c in it {
        let cur = c.into();

        if prev != cur {
            if let Some(pos) = is_not_space(prev, cur) {
                return Some(pos);
            }
        }

        prev = cur;
    }

    None
}

#[derive(Clone, Copy)]
struct Char {
    pos: usize,
    kind: CharKind,
}

impl PartialEq for Char {
    fn eq(&self, other: &Self) -> bool {
        self.kind.eq(&other.kind)
    }
}

impl From<(usize, char)> for Char {
    fn from(value: (usize, char)) -> Self {
        Self {
            pos: value.0,
            kind: value.1.into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CharKind {
    Space,
    Other,
}

impl From<char> for CharKind {
    fn from(ch: char) -> Self {
        if ch.is_whitespace() {
            Self::Space
        } else {
            Self::Other
        }
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

        shift_cursor_impl(&mut ws, Shift::Up(10));
        assert_eq!((0, 0), ws.curr().buf().pos());

        shift_cursor_impl(&mut ws, Shift::Bottom);
        assert_eq!((2, 0), ws.curr().buf().pos());

        shift_cursor_impl(&mut ws, Shift::Top);
        assert_eq!((0, 0), ws.curr().buf().pos());

        shift_cursor_impl(&mut ws, Shift::Right);
        assert_eq!((0, 1), ws.curr().buf().pos());

        shift_cursor_impl(&mut ws, Shift::Down(1));
        assert_eq!((1, 0), ws.curr().buf().pos());

        shift_cursor_impl(&mut ws, Shift::LineEnd);
        assert_eq!((1, 0), ws.curr().buf().pos());

        shift_cursor_impl(&mut ws, Shift::Left);
        assert_eq!((0, 4), ws.curr().buf().pos());

        shift_cursor_impl(&mut ws, Shift::Right);
        assert_eq!((1, 0), ws.curr().buf().pos());

        shift_cursor_impl(&mut ws, Shift::Right);
        assert_eq!((2, 0), ws.curr().buf().pos());

        shift_cursor_impl(&mut ws, Shift::LineEnd);
        assert_eq!((2, 3), ws.curr().buf().pos());

        shift_cursor_impl(&mut ws, Shift::Right);
        assert_eq!((2, 4), ws.curr().buf().pos());
    }
}
