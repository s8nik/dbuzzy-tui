pub enum Change {
    Insert(String),
    Delete(usize),
}

pub struct Commit {
    index: usize,
    offset: usize,
    change: Change,
}

impl Commit {
    pub fn insert() {}
    pub fn delete() {}
    pub fn apply() {}
}

pub struct History {
    index: usize,
    max_items: usize,
    commits: Vec<Commit>,
}
