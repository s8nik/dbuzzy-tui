use crate::buffer::{CursorMode, Position};

// 1. N
// history -> add('N', &buffer)
//  -

pub enum Change {
    Insert(String),
    Delete(String),
}

pub struct CommitData {
    pub mode: CursorMode,
    pub position: Position,
}

pub struct Commit {
    before: CommitData,
    after: CommitData,
    change: Change,
}

pub struct Transaction {
    current: Commit,
    inverted: Commit,
}

impl Transaction {
    pub fn apply() {}
}

#[derive(Default)]
pub struct History {
    index: usize,
    max_items: usize,
    commits: Vec<Transaction>,
}

impl History {
    pub fn new(max_items: usize) -> Self {
        Self {
            index: 0,
            max_items,
            commits: Vec::with_capacity(max_items),
        }
    }

    pub fn add() {}

    pub fn undo(text: &mut ropey::Rope) {}
    pub fn redo(text: &mut ropey::Rope) {}
}
