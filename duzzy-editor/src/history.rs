use std::collections::VecDeque;

use crate::buffer::Position;

#[derive(Debug, PartialEq, Eq)]
pub enum Change {
    Insert(String),
    Delete(String),
}

pub struct Commit {
    before: Position,
    after: Position,
    change: Change,
}

#[derive(Default)]
pub struct History {
    head: usize,
    max_items: usize,
    commits: VecDeque<Commit>,
}

impl History {
    pub fn new(max_items: usize) -> Self {
        Self {
            head: 0,
            max_items,
            commits: VecDeque::with_capacity(max_items),
        }
    }

    pub fn add(&mut self, commit: Commit) {
        if self.max_items == 0 {
            return;
        }

        match self.commits.back_mut() {
            Some(last) => match (&mut last.change, &commit.change) {
                (Change::Insert(prev), Change::Insert(change))
                | (Change::Delete(prev), Change::Delete(change)) => prev.push_str(change),
                _ => self.push(commit),
            },
            _ => self.push(commit),
        }
    }

    fn push(&mut self, commit: Commit) {
        if self.commits.len() == self.max_items {
            self.commits.pop_front();
            self.head = self.head.saturating_sub(1);
        }

        self.commits.push_back(commit);
        self.head += 1;
    }

    pub fn undo(text: &mut ropey::Rope) {}
    pub fn redo(text: &mut ropey::Rope) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history() {}
}
