use std::collections::VecDeque;

use crate::SmartString;

#[derive(Debug)]
pub enum ChangeKind {
    Insert,
    Delete,
}

#[derive(Debug)]
pub struct Change {
    kind: ChangeKind,
    content: SmartString,
    start: usize,
    end: usize,
}

impl Change {
    pub fn new_empty(kind: ChangeKind, pos: usize) -> Self {
        let content = SmartString::new_const();
        Self::new(kind, content, pos, pos)
    }

    pub fn new_insert(content: SmartString, start: usize, end: usize) -> Self {
        Self::new(ChangeKind::Insert, content, start, end)
    }

    pub fn new_delete(content: SmartString, start: usize, end: usize) -> Self {
        Self::new(ChangeKind::Delete, content, start, end)
    }

    pub fn on_char(&mut self, ch: char) {
        self.content.push(ch);
        self.update_state(1);
    }

    pub fn on_slice(&mut self, slice: &str) {
        self.content.push_str(slice);
        self.update_state(slice.chars().count())
    }

    fn update_state(&mut self, shift: usize) {
        match self.kind {
            ChangeKind::Insert => self.end += shift,
            ChangeKind::Delete => self.end -= shift,
        }
    }

    fn new(kind: ChangeKind, content: SmartString, start: usize, end: usize) -> Self {
        Self {
            kind,
            content,
            start,
            end,
        }
    }

    fn inverse(&self) -> Self {
        let kind = match self.kind {
            ChangeKind::Insert => ChangeKind::Delete,
            ChangeKind::Delete => ChangeKind::Insert,
        };

        let content = self.content.to_owned();

        Self {
            kind,
            content,
            start: self.start,
            end: self.end,
        }
    }

    fn apply(&self, text: &mut ropey::Rope) {
        match self.kind {
            ChangeKind::Insert => text.insert(self.start, &self.content),
            ChangeKind::Delete => {
                text.remove(self.start..self.start + self.content.chars().count())
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum ChangesResult {
    Commit,
    Keep,
}

pub struct History {
    head: usize,
    max_items: usize,
    changes: VecDeque<Change>,
    current: Option<Change>,
}

impl Default for History {
    fn default() -> Self {
        Self::new(Self::DEFAULT_CAPACITY)
    }
}

impl History {
    const DEFAULT_CAPACITY: usize = 20;

    pub fn new(max_items: usize) -> Self {
        Self {
            head: 0,
            max_items,
            current: None,
            changes: VecDeque::with_capacity(max_items),
        }
    }

    pub fn with_new_change<F>(&mut self, new_change: Change, func: F)
    where
        F: Fn(&mut Change) -> ChangesResult,
    {
        let mut current = match self.current.take() {
            Some(change) => change,
            None => new_change,
        };

        match func(&mut current) {
            ChangesResult::Commit => self.commit(current),
            ChangesResult::Keep => self.current = Some(current),
        }
    }

    fn commit(&mut self, change: Change) {
        if self.changes.len() == self.max_items {
            self.changes.pop_front();
            self.head = self.head.saturating_sub(1);
        }

        if self.head < self.changes.len() {
            self.changes.truncate(self.head);
        }

        self.changes.push_back(change);
        self.head += 1;
    }

    pub fn undo(&mut self, text: &mut ropey::Rope) -> Option<usize> {
        self.head = self.head.checked_sub(1)?;
        let change = &mut self.changes[self.head].inverse();

        change.apply(text);
        Some(change.start)
    }

    pub fn redo(&mut self, text: &mut ropey::Rope) -> Option<usize> {
        if self.head == self.changes.len() {
            return None;
        }

        let change = &mut self.changes[self.head];
        self.head += 1;

        change.apply(text);
        Some(change.end)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_history_delete_inplace() {
        let mut history = History::default();

        // test est
        let delete = Change::new_delete("t".into(), 5, 5);
        history.with_new_change(delete, |_| ChangesResult::Commit);

        // test st
        let delete = Change::new_delete("e".into(), 5, 5);
        history.with_new_change(delete, |_| ChangesResult::Commit);

        let mut text = ropey::Rope::from("test st");

        let pos = history.undo(&mut text).unwrap();
        assert_eq!(5, pos);
        assert_eq!(text.to_string().as_str(), "test est");

        let pos = history.undo(&mut text).unwrap();
        assert_eq!(5, pos);
        assert_eq!(text.to_string().as_str(), "test test");

        let pos = history.redo(&mut text).unwrap();
        assert_eq!(5, pos);
        assert_eq!(text.to_string().as_str(), "test est");

        let pos = history.redo(&mut text).unwrap();
        assert_eq!(5, pos);
        assert_eq!(text.to_string().as_str(), "test st");
    }

    #[test]
    fn test_history_on_char() {
        let mut history = History::default();

        let insert = Change::new_empty(ChangeKind::Insert, 0);
        history.with_new_change(insert, |change| {
            change.on_slice("test test");
            ChangesResult::Commit
        });

        let mut text = ropey::Rope::from_str("test test");

        let pos = history.undo(&mut text).unwrap();
        assert_eq!(0, pos);
        assert_eq!(text.to_string().as_str(), "");

        let pos = history.redo(&mut text).unwrap();
        assert_eq!(9, pos);
        assert_eq!(text.to_string().as_str(), "test test");
    }
}
