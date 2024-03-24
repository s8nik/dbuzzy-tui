use crate::SmartString;

#[derive(Debug)]
enum Action {
    Insert(Change),
    Delete(Change),
    Move,
}

impl Action {
    fn as_insert_mut(&mut self) -> Option<&mut Change> {
        match self {
            Self::Insert(change) => Some(change),
            _ => None,
        }
    }

    fn as_delete_mut(&mut self) -> Option<&mut Change> {
        match self {
            Self::Delete(change) => Some(change),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Change {
    content: SmartString,
    pub(super) pos: usize,
}

impl Change {
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
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

#[derive(Debug, Default)]
pub struct Transaction {
    changes: Vec<Action>,
}

impl Transaction {
    pub const fn new() -> Self {
        Self { changes: vec![] }
    }

    pub fn inverse(&self) -> Self {
        let changes = self.changes.iter().rev().map(Action::inverse).collect();

        Self { changes }
    }

    pub fn merge(&mut self, tx: Self) {
        for change in tx.changes {
            match change {
                Action::Insert(c) => self.insert_str(c.pos, &c.content),
                Action::Delete(c) => self.delete_str(c.pos, &c.content),
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

    pub fn insert_str(&mut self, pos: usize, slice: &str) {
        self.insert_impl(pos, |content| content.push_str(slice));
    }

    fn insert_impl<F>(&mut self, pos: usize, func: F)
    where
        F: Fn(&mut SmartString),
    {
        let maybe_change = self.changes.last_mut().and_then(|c| c.as_insert_mut());

        if let Some(change) = Self::merge_or_new(maybe_change, pos, func) {
            self.changes.push(Action::Insert(change));
        }
    }

    pub fn delete_char(&mut self, pos: usize, ch: char) {
        self.delete_impl(pos, |content| content.push(ch));
    }

    pub fn delete_str(&mut self, pos: usize, slice: &str) {
        let pos = pos.saturating_sub(slice.chars().count());
        self.delete_impl(pos, |content| content.push_str(slice));
    }

    fn delete_impl<F>(&mut self, pos: usize, func: F)
    where
        F: Fn(&mut SmartString),
    {
        let mut maybe_change = self.changes.last_mut().and_then(|c| c.as_delete_mut());

        if let Some(change) = maybe_change.as_deref_mut() {
            change.pos = pos;
        };

        if let Some(change) = Self::merge_or_new(maybe_change, pos, func) {
            self.changes.push(Action::Delete(change));
        }
    }

    fn merge_or_new<F>(maybe_change: Option<&mut Change>, pos: usize, func: F) -> Option<Change>
    where
        F: Fn(&mut SmartString),
    {
        match maybe_change {
            Some(change) => {
                func(&mut change.content);
                None
            }
            None => {
                let mut content = SmartString::new_const();
                func(&mut content);

                Some(Change { content, pos })
            }
        }
    }

    pub fn changes(&self) -> Vec<&Change> {
        self.changes
            .iter()
            .filter_map(|action| match action {
                Action::Insert(c) | Action::Delete(c) => Some(c),
                Action::Move => None,
            })
            .collect()
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
            tx.insert_char(0, 't');
            tx.insert_char(1, 'e');
            tx.insert_char(2, 's');
            tx.insert_char(3, 't');
            tx.apply(&mut text);

            assert_eq!(&text.to_string(), "test");
        }

        {
            let mut tx = Transaction::new();

            tx.delete_char(3, 't');
            tx.delete_char(2, 's');
            tx.shift();
            tx.shift();
            tx.insert_char(0, ' ');
            tx.shift();
            tx.insert_char(0, 't');
            tx.insert_char(1, 'e');
            tx.apply(&mut text);

            assert_eq!(&text.to_string(), "te te");
        }
    }
}
