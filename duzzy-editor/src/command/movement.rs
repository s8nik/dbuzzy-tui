use std::borrow::Cow;

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
    ByWord(ShiftWord),
}

#[derive(Copy, Clone, PartialEq, Eq)]
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
    buf.update_selection(buf.byte_pos());
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
    let text = buf.text();

    match kind {
        ShiftWord::PrevStart => shift_word_prev(text, idx, ofs),
        other => shift_word_next(other, text, idx, ofs),
    }
}

fn shift_word_next(kind: ShiftWord, text: &Rope, index: usize, offset: usize) -> Pos {
    let line = text.line(index);
    let len_lines = text.len_lines();
    let slice = line.slice(offset..);

    if let Some(ofs) = shift_word_next_impl(slice, kind, offset) {
        return (index, ofs);
    }

    if index + 1 < len_lines {
        return (index + 1, 0);
    }

    (index, line.chars().count())
}

fn shift_word_next_impl(slice: RopeSlice<'_>, kind: ShiftWord, offset: usize) -> Option<usize> {
    let mut it = slice.chars().enumerate();
    let mut prev: Char = it.next()?.into();

    for e in it {
        let cur: Char = e.into();

        let ch = match kind {
            ShiftWord::NextStart => cur,
            ShiftWord::NextEnd => prev,
            _ => unreachable!(),
        };

        if ch.kind != CharKind::Space && cur != prev && ch.pos != 0 {
            return Some(offset + ch.pos);
        }

        prev = cur;
    }

    None
}

fn shift_word_prev(text: &Rope, index: usize, offset: usize) -> Pos {
    let line = text.line(index);
    let slice = line.slice(..offset);

    if let Some(ofs) = shift_word_prev_impl(slice, offset) {
        return (index, ofs);
    }

    if index > 0 {
        return (index - 1, text.line(index - 1).chars().count() - 1);
    }

    (index, 0)
}

fn shift_word_prev_impl(slice: RopeSlice<'_>, offset: usize) -> Option<usize> {
    let slice = match slice.as_str() {
        Some(s) => Cow::from(s),
        None => Cow::from(slice.to_string()),
    };

    let mut it = slice.chars().rev().enumerate();
    let mut cur: Char = it.next()?.into();

    for e in it {
        let next: Char = e.into();

        if cur.kind != CharKind::Space && next != cur {
            return Some(offset - next.pos);
        }

        cur = next;
    }

    (cur.kind != CharKind::Space).then_some(0)
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

        let buf = ws.curr_mut().buf_mut();
        let text = Rope::from("test\n\ntest");
        buf.set_text(text);

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

    #[test]
    fn test_move_by_word() {
        let mut buf = Buffer::default();
        let text = Rope::from("test test test");
        buf.set_text(text);

        assert_eq!(shift_by_word(&mut buf, ShiftWord::NextStart), (0, 5));
        assert_eq!(shift_by_word(&mut buf, ShiftWord::NextEnd), (0, 3));

        buf.set_pos((0, 3));
        assert_eq!(shift_by_word(&mut buf, ShiftWord::NextEnd), (0, 8));

        buf.set_pos((0, 9));
        assert_eq!(shift_by_word(&mut buf, ShiftWord::PrevStart), (0, 5));

        let text = Rope::from(".te?/");
        buf.set_text(text);
        buf.set_pos((0, 0));

        assert_eq!(shift_by_word(&mut buf, ShiftWord::NextStart), (0, 1));
        assert_eq!(shift_by_word(&mut buf, ShiftWord::NextEnd), (0, 2));

        buf.set_pos((0, 4));
        assert_eq!(shift_by_word(&mut buf, ShiftWord::PrevStart), (0, 3));

        let text = Rope::from("test\n\n\ntest");
        buf.set_text(text);
        buf.set_pos((2, 0));
        assert_eq!(shift_by_word(&mut buf, ShiftWord::NextStart), (3, 0));
    }
}
