use std::collections::VecDeque;

use crate::{buffer::Position, SmartString};

#[derive(Debug)]
pub enum Change {
    Insert(SmartString),
    Delete(SmartString),
}

impl Change {
    fn inverse(&self) -> Self {
        match self {
            Self::Insert(c) => Self::Delete(c.to_owned()),
            Self::Delete(c) => Self::Insert(c.to_owned()),
        }
    }
}

#[derive(Debug)]
pub struct Commit {
    pub before: Position,
    pub after: Position,
    pub change: Change,
}

impl Commit {
    pub const fn new(change: Change, before: Position, after: Position) -> Self {
        Self {
            change,
            before,
            after,
        }
    }

    fn apply(change: &Change, text: &mut ropey::Rope, pos: Position) {
        let text_pos = pos.offset + text.line_to_byte(pos.index);

        match change {
            Change::Insert(c) => text.insert(text_pos, c),
            Change::Delete(c) => {
                dbg!(c, c.chars().count(), text_pos, text.len_chars());
                text.remove(text_pos..text_pos + c.chars().count());
            }
        }
    }

    fn undo(&self, text: &mut ropey::Rope) -> Position {
        Self::apply(&self.change.inverse(), text, self.before);
        self.before
    }

    fn redo(&self, text: &mut ropey::Rope) -> Position {
        Self::apply(&self.change, text, self.after);
        self.after
    }
}

pub struct History {
    head: usize,
    max_items: usize,
    commits: VecDeque<Commit>,
    transaction: Option<Commit>,
}

impl Default for History {
    fn default() -> Self {
        Self {
            head: 0,
            max_items: Self::DEFAULT_CAPACITY,
            transaction: None,
            commits: VecDeque::with_capacity(Self::DEFAULT_CAPACITY),
        }
    }
}

impl History {
    const DEFAULT_CAPACITY: usize = 20;

    pub fn tx(&mut self) -> Option<&mut Commit> {
        self.transaction.as_mut()
    }

    pub fn set_tx(&mut self, commit: Commit) {
        self.transaction = Some(commit);
    }

    pub fn commit(&mut self) {
        let Some(commit) = self.transaction.take() else {
            return;
        };

        if self.commits.len() == self.max_items {
            self.commits.pop_front();
            self.head = self.head.saturating_sub(1);
        }

        if self.head < self.commits.len() {
            self.commits.truncate(self.head);
        }

        self.commits.push_back(commit);
        self.head += 1;
    }

    pub fn undo(&mut self, text: &mut ropey::Rope) -> Option<Position> {
        self.head = self.head.checked_sub(1)?;
        let commit = &self.commits[self.head];

        Some(commit.undo(text))
    }

    pub fn redo(&mut self, text: &mut ropey::Rope) -> Option<Position> {
        if self.head == self.commits.len() {
            return None;
        }

        let commit = &self.commits[self.head];
        self.head += 1;

        Some(commit.redo(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_del_inplace() {
        //@note: let's say original text is "test test"

        let mut history = History::default();
        let before = (0, 5).into();
        let after = (0, 5).into();

        let commit = Commit::new(Change::Delete("t".into()), before, after);
        history.set_tx(commit);
        history.commit();
        // test est

        let commit = Commit::new(Change::Delete("e".into()), before, after);
        history.set_tx(commit);
        history.commit();
        // test st

        let mut text = ropey::Rope::from_str("test st");

        let pos = history.undo(&mut text).unwrap();
        assert_eq!((0, 5), Into::into(&pos));
        assert_eq!(text.to_string().as_str(), "test est");

        let pos = history.undo(&mut text).unwrap();
        assert_eq!((0, 5), Into::into(&pos));
        assert_eq!(text.to_string().as_str(), "test test");

        let pos = history.redo(&mut text).unwrap();
        assert_eq!((0, 5), Into::into(&pos));
        assert_eq!(text.to_string().as_str(), "test est");

        let pos = history.redo(&mut text).unwrap();
        assert_eq!((0, 5), Into::into(&pos));
        assert_eq!(text.to_string().as_str(), "test st");
    }

    #[test]
    fn test_history_insert() {
        //@note: let's say original text is "test test"

        let mut history = History::default();
        let before = (0, 0).into();
        let after = (0, 9).into();

        let commit = Commit::new(Change::Insert("test test".into()), before, after);
        history.set_tx(commit);
        history.commit();

        let mut text = ropey::Rope::from_str("test test");

        let pos = history.undo(&mut text).unwrap();
        assert_eq!((0, 0), Into::into(&pos));
        assert_eq!(text.to_string().as_str(), "");

        let pos = history.redo(&mut text).unwrap();
        assert_eq!((0, 9), Into::into(&pos));
        assert_eq!(text.to_string().as_str(), "test est");
    }
}
