use std::collections::VecDeque;

use crate::transaction::Transaction;

#[derive(Debug)]
pub struct History {
    head: usize,
    max_items: usize,
    commits: VecDeque<Transaction>,
}

impl Default for History {
    fn default() -> Self {
        Self::new(Self::DEFAULT_CAPACITY)
    }
}

impl History {
    const DEFAULT_CAPACITY: usize = 50;

    pub fn new(max_items: usize) -> Self {
        Self {
            head: 0,
            max_items,
            commits: VecDeque::with_capacity(max_items),
        }
    }

    pub fn commit(&mut self, tx: Transaction) {
        if self.commits.len() == self.max_items {
            self.commits.pop_front();
            self.head = self.head.saturating_sub(1);
        }

        if self.head < self.commits.len() {
            self.commits.truncate(self.head);
        }

        self.commits.push_back(tx);
        self.head += 1;
    }

    pub fn undo(&mut self, text: &mut ropey::Rope) -> Option<usize> {
        self.head = self.head.checked_sub(1)?;
        let commit = &mut self.commits[self.head];

        commit.inverse().apply(text)
    }

    pub fn redo(&mut self, text: &mut ropey::Rope) -> Option<usize> {
        if self.head == self.commits.len() {
            return None;
        }

        let commit = &mut self.commits[self.head];
        self.head += 1;

        commit.apply(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_undo_redo() {
        let mut history = History::default();
        let mut text = ropey::Rope::new();

        let mut tx = Transaction::new();
        tx.insert_str(0, "test");
        tx.apply(&mut text);
        history.commit(tx);
        assert_eq!(&text.to_string(), "test");

        let mut tx = Transaction::new();
        tx.insert_str(4, "test");
        tx.delete_str(0, "testtest");

        tx.apply(&mut text);
        history.commit(tx);
        assert_eq!(&text.to_string(), "");

        let pos = history.undo(&mut text);
        assert_eq!(Some(4), pos);
        assert_eq!(&text.to_string(), "test");

        let pos = history.redo(&mut text);
        assert_eq!(Some(0), pos);
        assert_eq!(&text.to_string(), "");
    }

    #[test]
    fn test_history_empty_commit() {
        let mut history = History::default();
        let mut text = ropey::Rope::new();

        let mut tx = Transaction::new();
        tx.insert_str(0, "test");
        tx.delete_char(3, 't');
        tx.delete_char(2, 's');
        tx.delete_char(1, 'e');
        tx.delete_char(0, 't');
        tx.apply(&mut text);

        history.commit(tx);

        let expected = text.to_string();

        let pos = history.undo(&mut text);
        assert_eq!(Some(0), pos);
        assert_eq!(&expected, "");

        let pos = history.redo(&mut text);
        assert_eq!(Some(0), pos);
        assert_eq!(&expected, "");
    }

    #[test]
    fn test_history_shifts() {
        let mut history = History::default();
        let mut text = ropey::Rope::from("test");

        let mut tx = Transaction::new();
        tx.shift(2);
        tx.insert_char(0, '\n');
        tx.shift(0);
        tx.insert_str(0, "test");
        tx.apply(&mut text);
        history.commit(tx);

        let pos = history.undo(&mut text);
        assert_eq!(Some(2), pos);
        assert_eq!(&text.to_string(), "test");

        let pos = history.redo(&mut text);
        assert_eq!(Some(4), pos);
        assert_eq!(&text.to_string(), "test\ntest");
    }
}
