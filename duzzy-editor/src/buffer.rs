use ropey::Rope;

#[derive(Debug, Default)]
pub struct Position {
    pub index: usize,
    pub offset: usize,
}

impl From<(usize, usize)> for Position {
    fn from(pos: (usize, usize)) -> Self {
        Self {
            index: pos.0,
            offset: pos.1,
        }
    }
}

impl Into<(usize, usize)> for &Position {
    fn into(self) -> (usize, usize) {
        (self.index, self.offset)
    }
}

#[derive(Debug)]
pub struct Buffer {
    pub(super) text: Rope,
    pub(super) pos: Position,
    pub(super) mode: CursorMode,

    vscroll: usize,
}

impl Buffer {
    pub fn text_pos(&self) -> usize {
        let (index, offset) = Into::into(&self.pos);
        offset + self.text.line_to_byte(index)
    }

    pub const fn vscroll(&self) -> usize {
        self.vscroll
    }

    pub fn update_vscroll(&mut self, max: usize) {
        let index = self.pos.index;
        let upper_bound = self.vscroll + max - 1;

        if index < self.vscroll {
            self.vscroll = index;
        } else if index > upper_bound {
            self.vscroll = index - max + 1;
        }
    }

    pub fn len_bytes(&self, index: usize) -> usize {
        self.text.line(index).len_bytes()
    }

    pub fn len_lines(&self) -> usize {
        self.text.len_lines()
    }

    pub fn is_insert(&self) -> bool {
        self.mode == CursorMode::Insert
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            text: Rope::default(),
            pos: Position::default(),
            vscroll: 0,
            mode: CursorMode::Normal,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CursorMode {
    Insert,
    Normal,
    Visual,
}

impl std::fmt::Display for CursorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Insert => write!(f, "insert"),
            Self::Normal => write!(f, "normal"),
            Self::Visual => write!(f, "visual"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::set_cursor;

    use super::Buffer;

    #[test]
    fn test_cursor_macro() {
        let mut buffer = Buffer::default();

        set_cursor!(buffer, index += 5);
        assert_eq!((5, 0), Into::into(&buffer.pos));

        set_cursor!(buffer, offset += 10);
        assert_eq!((5, 10), Into::into(&buffer.pos));

        set_cursor!(buffer, (15, 20).into());
        assert_eq!((15, 20), Into::into(&buffer.pos));
    }
}
