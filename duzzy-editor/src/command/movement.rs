use ropey::{Rope, RopeSlice};

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
    ByWord(ShiftWordKind),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ShiftWordKind {
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
    shift_cursor_impl(ws, Shift::ByWord(ShiftWordKind::NextEnd));
}

pub(super) fn move_next_word_start(ws: &mut Workspace) {
    shift_cursor_impl(ws, Shift::ByWord(ShiftWordKind::NextStart));
}

pub(super) fn move_prev_word_start(ws: &mut Workspace) {
    shift_cursor_impl(ws, Shift::ByWord(ShiftWordKind::PrevStart));
}

fn shift_cursor_impl(ws: &mut Workspace, shift: Shift) {
    let buf = ws.cur_mut().buf_mut();
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
        Shift::LineEnd => (idx, buf.line_len_bytes(idx).saturating_sub(1)),
        Shift::ByWord(kind) => shift_by_word(buf, kind),
    };

    buf.set_pos(pos);
    buf.update_selection(buf.byte_pos());
}

pub(super) fn shift_up(n: usize, buf: &mut Buffer) -> Pos {
    let (idx, ofs) = buf.pos();

    let idx = idx.saturating_sub(n);
    let ofs = ofs.min(buf.line_len_bytes(idx).saturating_sub(1));

    (idx, ofs)
}

pub(super) fn shift_down(n: usize, buf: &mut Buffer) -> Pos {
    let (idx, ofs) = buf.pos();

    let idx = (idx + n).min(buf.len_lines() - 1);
    let ofs = ofs.min(buf.line_len_bytes(idx).saturating_sub(1));

    (idx, ofs)
}

pub(super) fn shift_left(buf: &mut Buffer) -> Pos {
    let (idx, ofs) = buf.pos();

    match (ofs > 0, idx > 0) {
        (true, _) => (idx, ofs - 1),
        (false, true) => (idx - 1, buf.line_len_bytes(idx - 1).saturating_sub(1)),
        _ => (idx, ofs),
    }
}

pub(super) fn shift_right(buf: &mut Buffer) -> Pos {
    let (idx, ofs) = buf.pos();

    match (
        ofs < buf.line_len_bytes(idx).saturating_sub(1),
        idx < buf.len_lines().saturating_sub(1),
    ) {
        (true, _) => (idx, ofs + 1),
        (false, true) => ((idx + 1).min(buf.len_lines() - 1), 0),
        (false, false) => (idx, (ofs + 1).min(buf.line_len_bytes(idx))),
    }
}

fn shift_by_word(buf: &mut Buffer, kind: ShiftWordKind) -> Pos {
    let text = buf.text();
    let (idx, ofs) = buf.pos();

    let WordShiftPos { curs, sel } = match kind {
        ShiftWordKind::PrevStart => shift_word_backward(text, idx, ofs),
        _ => shift_word_forward(kind, text, idx, ofs),
    };

    if buf.selection().is_none() {
        if let Some(ofs) = sel {
            buf.new_selection(buf.line_byte(idx) + ofs);
        }
    }

    curs
}

fn shift_word_forward(kind: ShiftWordKind, text: &Rope, idx: usize, ofs: usize) -> WordShiftPos {
    let line = text.line(idx);
    let sel = shift_word_sel(kind, line, ofs);

    if let Some(shift_ofs) = shift_word_impl(kind, line, ofs) {
        let curs = (idx, shift_ofs);
        return WordShiftPos { curs, sel };
    }

    let len_lines = text.len_lines();
    let curs = if idx + 1 < len_lines {
        (idx + 1, 0)
    } else {
        (idx, line.chars().count() - 1)
    };

    WordShiftPos { curs, sel }
}

fn shift_word_backward(text: &Rope, idx: usize, ofs: usize) -> WordShiftPos {
    let line = text.line(idx);
    let kind = ShiftWordKind::PrevStart;

    let ofs = if ofs != 0 {
        (ofs + 1).min(line.len_chars())
    } else {
        ofs
    };

    if let Some(shift_ofs) = shift_word_impl(kind, line, ofs) {
        return WordShiftPos {
            curs: (idx, shift_ofs),
            sel: shift_word_sel(kind, line, ofs),
        };
    }

    let curs = if idx > 0 {
        (idx - 1, text.line(idx - 1).chars().count() - 1)
    } else {
        (idx, 0)
    };

    WordShiftPos { curs, sel: None }
}

fn shift_word_impl(kind: ShiftWordKind, line: RopeSlice<'_>, ofs: usize) -> Option<usize> {
    let mut iter = shift_word_chs(kind, line, ofs);
    let mut prev: Char = iter.next()?.into();

    for ch in iter {
        let next: Char = ch.into();
        let (cur, res) = shift_word_cur(kind, prev, next);

        if cur.kind != CharKind::Space && next != prev && prev.pos != 0 {
            let pos = res.map(|r| r.pos).unwrap_or(cur.pos);
            return Some(shift_word_ofs(kind, ofs, pos));
        }

        prev = next;
    }

    shift_word_def(kind, prev)
}

fn shift_word_sel(kind: ShiftWordKind, line: RopeSlice<'_>, ofs: usize) -> Option<usize> {
    let mut iter = shift_word_chs(kind, line, ofs);

    let prev: Char = iter.next()?.into();
    let next: Char = iter.next()?.into();

    let pos = if prev.kind != CharKind::Space && next != prev {
        next.pos
    } else {
        prev.pos
    };

    Some(shift_word_ofs(kind, ofs, pos))
}

fn shift_word_chs(
    kind: ShiftWordKind,
    line: RopeSlice<'_>,
    ofs: usize,
) -> std::iter::Enumerate<ropey::iter::Chars<'_>> {
    match kind {
        ShiftWordKind::PrevStart => line.chars_at(ofs).reversed().enumerate(),
        _ => line.slice(ofs..).chars().enumerate(),
    }
}

fn shift_word_def(kind: ShiftWordKind, last: Char) -> Option<usize> {
    match kind {
        ShiftWordKind::PrevStart => (last.kind != CharKind::Space).then_some(0),
        _ => None,
    }
}

const fn shift_word_ofs(kind: ShiftWordKind, ofs: usize, ch_ofs: usize) -> usize {
    match kind {
        ShiftWordKind::PrevStart => ofs - ch_ofs,
        _ => ofs + ch_ofs,
    }
}

const fn shift_word_cur(kind: ShiftWordKind, prev: Char, next: Char) -> (Char, Option<Char>) {
    match kind {
        ShiftWordKind::NextStart => (next, Some(prev)),
        ShiftWordKind::PrevStart => (prev, Some(next)),
        ShiftWordKind::NextEnd => (prev, None),
    }
}

struct WordShiftPos {
    curs: Pos,
    sel: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharKind {
    Space,
    Punct,
    Other,
}

impl From<char> for CharKind {
    fn from(ch: char) -> Self {
        if ch.is_whitespace() {
            Self::Space
        } else if ch.is_ascii_punctuation() {
            Self::Punct
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

        let buf = ws.cur_mut().buf_mut();
        let text = Rope::from("test\n\ntest");
        buf.set_text(text);

        shift_cursor_impl(&mut ws, Shift::Up(10));
        assert_eq!((0, 0), ws.cur().buf().pos());

        shift_cursor_impl(&mut ws, Shift::Bottom);
        assert_eq!((2, 0), ws.cur().buf().pos());

        shift_cursor_impl(&mut ws, Shift::Top);
        assert_eq!((0, 0), ws.cur().buf().pos());

        shift_cursor_impl(&mut ws, Shift::Right);
        assert_eq!((0, 1), ws.cur().buf().pos());

        shift_cursor_impl(&mut ws, Shift::Down(1));
        assert_eq!((1, 0), ws.cur().buf().pos());

        shift_cursor_impl(&mut ws, Shift::LineEnd);
        assert_eq!((1, 0), ws.cur().buf().pos());

        shift_cursor_impl(&mut ws, Shift::Left);
        assert_eq!((0, 4), ws.cur().buf().pos());

        shift_cursor_impl(&mut ws, Shift::Right);
        assert_eq!((1, 0), ws.cur().buf().pos());

        shift_cursor_impl(&mut ws, Shift::Right);
        assert_eq!((2, 0), ws.cur().buf().pos());

        shift_cursor_impl(&mut ws, Shift::LineEnd);
        assert_eq!((2, 3), ws.cur().buf().pos());

        shift_cursor_impl(&mut ws, Shift::Right);
        assert_eq!((2, 4), ws.cur().buf().pos());
    }

    #[test]
    fn test_move_by_word() {
        let mut buf = Buffer::default();
        let text = Rope::from("test test test");
        buf.set_text(text);

        assert_eq!(shift_by_word(&mut buf, ShiftWordKind::NextStart), (0, 4));
        assert_eq!(shift_by_word(&mut buf, ShiftWordKind::NextEnd), (0, 3));

        buf.set_pos((0, 9));
        assert_eq!(shift_by_word(&mut buf, ShiftWordKind::NextStart), (0, 13));

        buf.set_pos((0, 3));
        assert_eq!(shift_by_word(&mut buf, ShiftWordKind::NextEnd), (0, 8));

        buf.set_pos((0, 9));
        assert_eq!(shift_by_word(&mut buf, ShiftWordKind::PrevStart), (0, 5));

        let text = Rope::from(".te?/");
        buf.set_text(text);
        buf.set_pos((0, 0));

        assert_eq!(shift_by_word(&mut buf, ShiftWordKind::NextStart), (0, 2));
        assert_eq!(shift_by_word(&mut buf, ShiftWordKind::NextEnd), (0, 2));

        buf.set_pos((0, 4));
        assert_eq!(shift_by_word(&mut buf, ShiftWordKind::PrevStart), (0, 3));

        let text = Rope::from("test\n\n\ntest");
        buf.set_text(text);
        buf.set_pos((2, 0));
        assert_eq!(shift_by_word(&mut buf, ShiftWordKind::NextStart), (3, 0));
    }
}
