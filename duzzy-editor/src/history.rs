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
    pub const fn new(kind: ChangeKind, pos: usize) -> Self {
        Self {
            kind,
            content: SmartString::new_const(),
            start: pos,
            end: pos,
        }
    }

    pub fn on_char(&mut self, ch: char, inplace: bool) -> &mut Self {
        self.content.push(ch);

        if !inplace {
            self.update_state(1);
        }

        self
    }

    pub fn on_slice(&mut self, slice: &str) -> &mut Self {
        self.content.push_str(slice);
        self.update_state(slice.chars().count());

        self
    }

    pub const fn commit(&self) -> ChangesResult {
        ChangesResult::Commit
    }

    pub const fn keep(&self) -> ChangesResult {
        ChangesResult::Keep
    }

    fn update_state(&mut self, shift: usize) {
        match self.kind {
            ChangeKind::Insert => self.end += shift,
            ChangeKind::Delete => self.end -= shift,
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
        let pos = self.start.min(self.end);
        let reverted: Option<SmartString> =
            (self.start > self.end).then_some(self.content.chars().rev().collect());

        let content = reverted
            .as_ref()
            .map(|c| c.as_str())
            .unwrap_or(self.content.as_str());

        match self.kind {
            ChangeKind::Insert => text.insert(pos, content),
            ChangeKind::Delete => text.remove(pos..pos + content.chars().count()),
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

    pub fn push<F>(&mut self, kind: ChangeKind, pos: usize, func: F)
    where
        F: Fn(&mut Change) -> ChangesResult,
    {
        let mut current = match self.current.take() {
            Some(change) => change,
            None => Change::new(kind, pos),
        };

        match func(&mut current) {
            ChangesResult::Commit => self.commit_impl(current),
            ChangesResult::Keep => self.current = Some(current),
        }
    }

    pub fn commit(&mut self) {
        if let Some(change) = self.current.take() {
            self.commit_impl(change);
        }
    }

    fn commit_impl(&mut self, change: Change) {
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
        history.push(ChangeKind::Delete, 5, |change| {
            change.on_char('t', true).commit()
        });

        // test st
        history.push(ChangeKind::Delete, 5, |change| {
            change.on_char('e', true).commit()
        });

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

        history.push(ChangeKind::Delete, 9, |change| {
            change
                .on_char('t', false)
                .on_char('s', false)
                .on_char('e', false)
                .on_char('t', false)
                .commit()
        });

        let mut text = ropey::Rope::from_str("test ");

        let pos = history.undo(&mut text).unwrap();
        assert_eq!(9, pos);
        assert_eq!(text.to_string().as_str(), "test test");

        let pos = history.redo(&mut text).unwrap();
        assert_eq!(5, pos);
        assert_eq!(text.to_string().as_str(), "test ");
    }

    #[test]
    fn test_history_on_slice() {
        let mut history = History::default();

        history.push(ChangeKind::Insert, 0, |change| {
            change.on_slice("test test").commit()
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
