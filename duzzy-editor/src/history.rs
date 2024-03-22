use std::collections::VecDeque;

use crate::SmartString;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Insert,
    Delete,
}

#[derive(Debug)]
pub(super) struct Change {
    action: Action,
    content: SmartString,
    start: usize,
    end: usize,
}

impl Change {
    const fn new(action: Action, pos: usize) -> Self {
        Self {
            action,
            content: SmartString::new_const(),
            start: pos,
            end: pos,
        }
    }

    fn inverse(&self) -> Self {
        let (action, content) = match self.action {
            Action::Insert => (Action::Delete, self.content.to_owned()),
            Action::Delete => (Action::Insert, self.content.chars().rev().collect()),
        };

        Self {
            action,
            content,
            start: self.start,
            end: self.end,
        }
    }

    fn apply(&self, text: &mut ropey::Rope) {
        let pos = self.start.min(self.end);
        let content = &self.content;

        match self.action {
            Action::Insert => text.insert(pos, content),
            Action::Delete => text.remove(pos..pos + content.chars().count()),
        }
    }
}

#[derive(Debug)]
pub struct Transaction {
    action: Action,
    change: Change,
}

impl Transaction {
    const fn new(action: Action, pos: usize) -> Self {
        let change = Change::new(action, pos);

        Self { action, change }
    }

    const fn update_action(mut self, action: Action) -> Self {
        self.action = action;
        self
    }

    pub fn on_char(mut self, ch: char, inplace: bool) -> Self {
        if !inplace {
            self.update_state(1);
        }

        let is_same = self.change.action == self.action;
        let content = &mut self.change.content;

        if is_same {
            content.push(ch);
        } else {
            let idx = content.chars().count() - 1;
            content.remove(idx);
        }

        self
    }

    pub fn on_slice(mut self, slice: &str) -> Self {
        let slice_len = slice.chars().count();
        self.update_state(slice_len);

        let is_same = self.change.action == self.action;
        let content = &mut self.change.content;

        if is_same {
            content.push_str(slice);
        } else if let Some(idx) = content.rfind(slice) {
            content.replace_range(idx..idx + slice_len, "");
        }

        self
    }

    pub fn commit(self) -> TransactionResult {
        TransactionResult::Commit(self.finish())
    }

    pub const fn keep(self) -> TransactionResult {
        TransactionResult::Keep(self)
    }

    fn finish(self) -> Option<Change> {
        let change = self.change;

        (!change.content.is_empty()).then_some(change)
    }

    fn update_state(&mut self, shift: usize) {
        match self.action {
            Action::Insert => self.change.end += shift,
            Action::Delete => self.change.end -= shift,
        }
    }
}

pub enum TransactionResult {
    Commit(Option<Change>),
    Keep(Transaction),
}

pub struct History {
    head: usize,
    max_items: usize,
    changes: VecDeque<Change>,
    transaction: Option<Transaction>,
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
            transaction: None,
            changes: VecDeque::with_capacity(max_items),
        }
    }

    pub fn push<F>(&mut self, action: Action, pos: usize, func: F)
    where
        F: Fn(Transaction) -> TransactionResult,
    {
        let tx = match self.transaction.take() {
            Some(transaction) => transaction.update_action(action),
            None => Transaction::new(action, pos),
        };

        match func(tx) {
            TransactionResult::Commit(change) => self.maybe_commit(change),
            TransactionResult::Keep(tx) => self.transaction = Some(tx),
        }
    }

    pub fn commit(&mut self) {
        if let Some(tx) = self.transaction.take() {
            self.maybe_commit(tx.finish());
        }
    }

    fn maybe_commit(&mut self, change: Option<Change>) {
        let Some(change) = change else {
            self.head = self.head.saturating_sub(1);
            return;
        };

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
        history.push(Action::Delete, 5, |tx| tx.on_char('t', true).commit());

        // test st
        history.push(Action::Delete, 5, |tx| tx.on_char('e', true).commit());

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

        history.push(Action::Delete, 9, |tx| {
            tx.on_char('t', false)
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

        history.push(Action::Insert, 0, |tx| tx.on_slice("test test").commit());

        let mut text = ropey::Rope::from_str("test test");

        let pos = history.undo(&mut text).unwrap();
        assert_eq!(0, pos);
        assert_eq!(text.to_string().as_str(), "");

        let pos = history.redo(&mut text).unwrap();
        assert_eq!(9, pos);
        assert_eq!(text.to_string().as_str(), "test test");
    }

    #[test]
    fn test_history_on_slice_empty_commit() {
        let mut history = History::default();

        history.push(Action::Insert, 0, |tx| tx.on_slice("test").keep());

        history.push(Action::Delete, 4, |tx| tx.on_slice("test").commit());

        let mut text = ropey::Rope::new();
        let expected = text.to_string();

        let pos = history.undo(&mut text);
        assert_eq!(None, pos);
        assert_eq!(&expected, "");

        let pos = history.redo(&mut text);
        assert_eq!(None, pos);
        assert_eq!(&expected, "");
    }

    #[test]
    fn test_history_on_char_empty_commit() {
        let mut history = History::default();

        history.push(Action::Insert, 0, |tx| {
            tx.on_char('t', false)
                .on_char('e', false)
                .on_char('s', false)
                .on_char('t', false)
                .keep()
        });

        history.push(Action::Delete, 4, |tx| tx.on_char('t', false).keep());
        history.push(Action::Delete, 3, |tx| tx.on_char('s', false).keep());
        history.push(Action::Delete, 2, |tx| tx.on_char('e', false).keep());
        history.push(Action::Delete, 1, |tx| tx.on_char('t', false).keep());
        history.commit();

        let mut text = ropey::Rope::new();
        let expected = text.to_string();

        let pos = history.undo(&mut text);
        assert_eq!(None, pos);
        assert_eq!(&expected, "");

        let pos = history.redo(&mut text);
        assert_eq!(None, pos);
        assert_eq!(&expected, "");
    }

    #[test]
    fn test_history_transaction() {
        let mut history = History::default();

        history.push(Action::Insert, 0, |tx| {
            tx.on_slice("test")
                .on_char('\n', false)
                .on_char('\n', false)
                .on_slice("test!!!")
                .keep()
        });

        history.push(Action::Delete, 0, |tx| {
            tx.on_char('!', false)
                .on_char('!', false)
                .on_char('!', false)
                .keep()
        });

        history.commit();

        let mut text = ropey::Rope::from_str("test\n\ntest");

        let pos = history.undo(&mut text);
        assert_eq!(Some(0), pos);
        assert_eq!(&text.to_string(), "");

        let pos = history.redo(&mut text);
        assert_eq!(Some(10), pos);
        assert_eq!(&text.to_string(), "test\n\ntest");
    }
}
