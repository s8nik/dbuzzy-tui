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
    let doc = ws.cur_mut();
    let buf = doc.buf_mut();

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
        Shift::LineEnd => shift_line_end(buf),
        Shift::ByWord(kind) => shift_by_word(buf, kind),
    };

    buf.set_pos(pos);

    let byte_pos = buf.byte_pos();
    let is_insert = buf.is_insert();

    buf.update_selection(byte_pos);

    if let Some(tx) = doc.transaction() {
        if is_insert {
            tx.shift(byte_pos);
        }
    }
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

fn shift_line_end(buf: &mut Buffer) -> Pos {
    let idx = buf.index();
    let shift = if idx == buf.len_lines() - 1 { 1 } else { 2 };
    (idx, buf.line_len_bytes(idx).saturating_sub(shift))
}

fn shift_by_word(buf: &mut Buffer, kind: ShiftWordKind) -> Pos {
    let shift = ShiftWord::new(buf);

    let (cursor, anchor) = match kind {
        ShiftWordKind::PrevStart => shift.backward(),
        other => shift.forward(other),
    };

    if !buf.is_selection() {
        if let Some(ofs) = anchor {
            buf.new_selection(buf.line_byte(buf.index()) + ofs);
        }
    }

    cursor
}

type ShiftWordPos = (Pos, Option<usize>);

struct ShiftWord<'a> {
    text: &'a Rope,
    line: RopeSlice<'a>,
    idx: usize,
    ofs: usize,
}

impl<'a> ShiftWord<'a> {
    fn new(buf: &'a Buffer) -> Self {
        let text = buf.text();
        let (idx, ofs) = buf.pos();
        let line = text.line(idx);

        Self {
            text,
            line,
            idx,
            ofs,
        }
    }

    fn end_chunk(prev: Char, next: Char) -> bool {
        prev.kind != CharKind::Space && next != prev
    }

    fn selection_anchor(prev: Char, next: Char) -> usize {
        if Self::end_chunk(prev, next) {
            next.pos
        } else {
            prev.pos
        }
    }

    fn forward(&self, kind: ShiftWordKind) -> ShiftWordPos {
        if let Some((ofs, anchor)) = Self::forward_impl(self.line, self.ofs, kind) {
            let cursor = (self.idx, ofs);
            return (cursor, Some(anchor));
        }

        let cursor = if self.idx + 1 < self.text.len_lines() {
            (self.idx + 1, 0)
        } else {
            (self.idx, self.line.chars().count() - 1)
        };

        (cursor, None)
    }

    fn forward_impl(
        line: RopeSlice<'a>,
        ofs: usize,
        kind: ShiftWordKind,
    ) -> Option<(usize, usize)> {
        let mut it = line.slice(ofs..).chars().enumerate();

        let mut prev: Char = it.next()?.into();
        let next: Char = it.next()?.into();

        let mut anch = ofs + Self::selection_anchor(prev, next);
        if kind == ShiftWordKind::NextStart && (prev.kind == CharKind::Space && next != prev) {
            anch += 1;
        }

        prev = next;
        for ch in it {
            let next: Char = ch.into();

            let (p, n) = match kind {
                ShiftWordKind::NextStart => (next, prev),
                ShiftWordKind::NextEnd => (prev, next),
                ShiftWordKind::PrevStart => return None,
            };

            if Self::end_chunk(p, n) {
                let ofs = ofs + prev.pos;
                return Some((ofs, anch));
            }

            prev = next;
        }

        let len_chars = line.len_chars();
        (ofs != len_chars).then_some((len_chars - 1, anch))
    }

    fn backward(&self) -> ShiftWordPos {
        if let Some((ofs, anchor)) = Self::backward_impl(self.line, self.ofs) {
            let cursor = (self.idx, ofs);
            return (cursor, Some(anchor));
        }

        let idx = self.idx;
        let text = self.text;

        let cursor = if idx > 0 {
            (idx - 1, text.line(idx - 1).chars().count() - 1)
        } else {
            (idx, 0)
        };

        (cursor, None)
    }

    fn backward_impl(line: RopeSlice<'a>, mut ofs: usize) -> Option<(usize, usize)> {
        let len_chars = line.len_chars();

        let mut it = line
            .chars_at((ofs + 1).min(len_chars))
            .reversed()
            .enumerate();

        let mut anch = ofs;
        let mut prev: Char = it.next()?.into();

        for ch in it {
            let next: Char = ch.into();

            if prev.pos == 0 {
                anch -= Self::selection_anchor(prev, next);

                if ofs != len_chars {
                    prev = next;
                    continue;
                }

                ofs -= 1;
            }

            if Self::end_chunk(prev, next) {
                let ofs = ofs - prev.pos;
                return Some((ofs, anch));
            }

            prev = next;
        }

        (ofs != 0).then_some((0, anch))
    }
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

        let text = Rope::from("test t");
        buf.set_text(text);
        buf.set_pos((0, 6));
        assert_eq!(shift_by_word(&mut buf, ShiftWordKind::PrevStart), (0, 5));

        buf.set_pos((0, 5));
        assert_eq!(shift_by_word(&mut buf, ShiftWordKind::PrevStart), (0, 0));
    }
}
