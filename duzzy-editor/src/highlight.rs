#[derive(Debug, Clone, Copy)]
pub struct Selection {
    anchor: usize,
    head: usize,
}

impl Selection {
    pub const fn new(pos: usize) -> Self {
        Self {
            anchor: pos,
            head: pos,
        }
    }

    pub fn start(self) -> usize {
        self.head.min(self.anchor)
    }

    pub fn end(self) -> usize {
        self.head.max(self.anchor)
    }

    pub fn update(&mut self, pos: usize) {
        self.head = pos;
    }
}
