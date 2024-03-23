use crate::{history::Commit, SmartString};

#[derive(Debug)]
enum Action {
    Insert(Change),
    Delete(Change),
    Move,
}

#[derive(Debug)]
struct Change {
    content: SmartString,
    pos: usize,
}

impl Action {
    fn inverse(&self) -> Self {
        match self {
            Self::Insert(change) => Self::Delete(Change {
                content: change.content.to_owned(),
                pos: change.pos,
            }),
            Self::Delete(change) => {
                let content = change.content.chars().rev().collect();

                Self::Insert(Change {
                    content,
                    pos: change.pos,
                })
            }
            Self::Move => Self::Move,
        }
    }
}

macro_rules! merge {
    () => {};
}

#[derive(Debug, Default)]
pub struct Transaction {
    changes: Vec<Action>,
}

impl Transaction {
    pub const fn new() -> Self {
        Self { changes: vec![] }
    }

    pub fn inverse(&self) -> Self {
        let changes = self.changes.iter().map(Action::inverse).collect();

        Self { changes }
    }

    pub fn merge(&mut self, tx: Transaction) {
        for change in tx.changes {
            match change {
                Action::Insert(c) => {
                    self.insert_impl(c.pos, |content| content.push_str(&c.content))
                }
                Action::Delete(c) => {
                    self.delete_impl(c.pos, |content| content.push_str(&c.content))
                }
                Action::Move => self.shift(),
            }
        }
    }

    pub fn apply(&self, text: &mut ropey::Rope) {
        for change in self.changes.iter() {
            match change {
                Action::Insert(c) => text.insert(c.pos, &c.content),
                Action::Delete(c) => text.remove(c.pos..c.pos + c.content.chars().count()),
                _ => (),
            }
        }
    }

    pub fn shift(&mut self) {
        if !matches!(self.changes.last(), Some(Action::Move)) {
            self.changes.push(Action::Move);
        }
    }

    pub fn insert_char(&mut self, pos: usize, ch: char) {
        self.insert_impl(pos, |content| content.push(ch));
    }

    fn insert_impl<F>(&mut self, pos: usize, func: F)
    where
        F: Fn(&mut SmartString),
    {
        if let Some(Action::Insert(change)) = self.changes.last_mut() {
            return func(&mut change.content);
        }

        let mut content = SmartString::new_const();
        func(&mut content);

        self.changes.push(Action::Insert(Change { content, pos }));
    }

    pub fn delete_char(&mut self, pos: usize, ch: char) {
        self.delete_impl(pos, |content| content.push(ch));
    }

    fn delete_impl<F>(&mut self, pos: usize, func: F)
    where
        F: Fn(&mut SmartString),
    {
        if let Some(Action::Delete(change)) = self.changes.last_mut() {
            change.pos = pos;
            return func(&mut change.content);
        }

        let mut content = SmartString::new_const();
        func(&mut content);

        self.changes.push(Action::Delete(Change { content, pos }));
    }
}

impl From<Transaction> for Commit {
    fn from(tx: Transaction) -> Self {
        let positions: Vec<usize> = tx
            .changes
            .iter()
            .filter_map(|action| match action {
                Action::Insert(change) | Action::Delete(change) => Some(change.pos),
                Action::Move => None,
            })
            .collect();

        let before = *positions.first().unwrap();
        let after = *positions.last().unwrap();

        Self { tx, before, after }
    }
}

pub enum TransactionResult {
    Commit,
    Keep,
    Abort,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction() {
        let mut text = ropey::Rope::new();

        {
            let mut tx = Transaction::new();
            tx.insert_char(0, 't')
                .insert_char(1, 'e')
                .insert_char(2, 's')
                .insert_char(3, 't')
                .apply(&mut text);

            assert_eq!(&text.to_string(), "test");
        }

        {
            let mut tx = Transaction::new();

            tx.delete_char(3, 't')
                .delete_char(2, 's')
                .shift()
                .shift()
                .insert_char(0, ' ')
                .shift()
                .insert_char(0, 't')
                .insert_char(1, 'e')
                .apply(&mut text);

            assert_eq!(&text.to_string(), "te te");
        }
    }
}
