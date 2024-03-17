use std::collections::VecDeque;

use crate::buffer::Position;

pub enum Action {
    InsertChar(char),
    DeleteChar(char),
}

#[derive(Debug)]
enum Change {
    Insert(String),
    Delete(String),
}

impl Change {
    fn inverse(&self) -> Self {
        match self {
            Change::Insert(c) => Change::Delete(c.to_owned()),
            Change::Delete(c) => Change::Insert(c.to_owned()),
        }
    }
}

impl From<Action> for Change {
    fn from(action: Action) -> Self {
        match action {
            Action::InsertChar(ch) => Self::Insert(String::from(ch)),
            Action::DeleteChar(ch) => Self::Delete(String::from(ch)),
        }
    }
}

#[derive(Debug)]
struct Commit {
    before: Position,
    after: Position,
    change: Change,
    inverse: Change,
}

impl Commit {
    fn new(change: Change, before: Position, after: Position) -> Self {
        let inverse = change.inverse();

        Self {
            change,
            inverse,
            before,
            after,
        }
    }

    fn change(&mut self, action: Action, before: Position, after: Position) -> Option<Self> {
        let mut new_kind = None;

        match (&mut self.change, &mut self.inverse, &action) {
            (Change::Insert(c), Change::Delete(i), Action::InsertChar(ch))
            | (Change::Delete(c), Change::Insert(i), Action::DeleteChar(ch)) => {
                c.push(*ch);
                i.push(*ch);
            }
            _ => new_kind = Some(action.into()),
        };

        if let Some(kind) = new_kind.take() {
            return Some(Self::new(kind, before, after));
        }

        self.after = after;
        None
    }

    fn apply(change: &Change, text: &mut ropey::Rope, pos: Position) {
        let text_pos = pos.offset + text.line_to_byte(pos.index);

        match change {
            Change::Insert(c) => text.insert(text_pos, c),
            Change::Delete(c) => text.remove(text_pos - c.chars().count()..text_pos),
        }
    }

    pub fn undo(&self, text: &mut ropey::Rope) -> Position {
        Self::apply(&self.inverse, text, self.after);
        self.before
    }

    pub fn redo(&self, text: &mut ropey::Rope) -> Position {
        Self::apply(&self.change, text, self.before);
        self.after
    }
}

#[derive(Default)]
pub struct History {
    head: usize,
    max_items: usize,
    commits: VecDeque<Commit>,
    transaction: Option<Commit>,
}

impl History {
    pub fn new(max_items: usize) -> anyhow::Result<Self> {
        anyhow::ensure!(max_items != 0, "invalid max length");

        Ok(Self {
            head: 0,
            max_items,
            transaction: None,
            commits: VecDeque::with_capacity(max_items),
        })
    }

    pub fn commit(&mut self, action: Action, before: Position, after: Position) {
        let mut maybe_new = match self.transaction.as_mut() {
            Some(commit) => commit.change(action, before, after),
            None => Some(Commit::new(action.into(), before, after)),
        };

        if let Some(commit) = maybe_new.take() {
            self.apply();
            self.transaction = Some(commit);
        }
    }

    pub fn apply(&mut self) {
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
    fn test_history() {
        //@note: let's say original text is "hello world!"

        let mut history = History::new(10).unwrap();
        let before = (0, 5).into();
        let mut after = (0, 6).into();

        history.commit(Action::InsertChar(' '), before, after);
        after.offset += 1;
        history.commit(Action::InsertChar(' '), before, after);
        history.apply();
        // hello   world!

        let before = (0, 6).into();
        let after = (0, 7).into();
        history.commit(Action::InsertChar('o'), before, after);
        history.apply();
        // hello o  world!

        let before = (0, 7).into();
        let after = (0, 8).into();
        history.commit(Action::InsertChar('k'), before, after);
        history.apply();
        // hello ok  world!

        let before = (0, 10).into();
        let mut after = (0, 9).into();
        history.commit(Action::DeleteChar(' '), before, after);
        after.offset -= 1;
        history.commit(Action::DeleteChar(' '), before, after);
        history.apply();
        // hello okworld!

        let mut text = ropey::Rope::from_str("hello okworld!");

        let pos = history.undo(&mut text).unwrap();
        assert_eq!((0, 10), Into::into(&pos));
        assert_eq!(text.to_string().as_str(), "hello ok  world!");

        let pos = history.undo(&mut text).unwrap();
        assert_eq!((0, 7), Into::into(&pos));
        assert_eq!(text.to_string().as_str(), "hello o  world!");

        let pos = history.undo(&mut text).unwrap();
        assert_eq!((0, 6), Into::into(&pos));
        assert_eq!(text.to_string().as_str(), "hello   world!");

        let pos = history.undo(&mut text).unwrap();
        assert_eq!((0, 5), Into::into(&pos));
        assert_eq!(text.to_string().as_str(), "hello world!");

        let pos = history.redo(&mut text).unwrap();
        assert_eq!((0, 7), Into::into(&pos));
        assert_eq!(text.to_string().as_str(), "hello   world!");

        let pos = history.redo(&mut text).unwrap();
        assert_eq!((0, 7), Into::into(&pos));
        assert_eq!(text.to_string().as_str(), "hello o  world!");

        let pos = history.redo(&mut text).unwrap();
        assert_eq!((0, 8), Into::into(&pos));
        assert_eq!(text.to_string().as_str(), "hello ok  world!");

        let pos = history.redo(&mut text).unwrap();
        assert_eq!((0, 8), Into::into(&pos));
        assert_eq!(text.to_string().as_str(), "hello okworld!");
    }
}
