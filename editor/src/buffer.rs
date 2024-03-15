use ropey::Rope;

#[derive(Debug, Default)]
pub struct Position {
    pub index: usize,
    pub offset: usize,
}

impl From<(usize, usize)> for Position {
    fn from(value: (usize, usize)) -> Self {
        Self {
            index: value.0,
            offset: value.1,
        }
    }
}

#[derive(Debug)]
pub struct Buffer {
    pub(super) text: Rope,
    pub(super) position: Position,
    pub(super) mode: CursorMode,

    vscroll: usize,
}

#[macro_export]
macro_rules! cursor {
    ($buffer:expr) => {{
        ($buffer.position.index, $buffer.position.offset)
    }};
    ($buffer:expr, index) => {{
        $buffer.position.index
    }};
    ($buffer:expr, offset) => {{
        $buffer.position.offset
    }};
    ($buffer:expr, index $op:tt $value:expr) => {{
        match stringify!($op) {
            "=" => $buffer.position.index = $value,
            "+=" => $buffer.position.index += $value,
            "-=" => $buffer.position.index -= $value,
            _ => unreachable!(),
        };
    }};
    ($buffer:expr, offset $op:tt $value:expr) => {{
        match stringify!($op) {
            "=" => $buffer.position.offset = $value,
            "+=" => $buffer.position.offset += $value,
            "-=" => $buffer.position.offset -= $value,
            _ => unreachable!(),
        };
    }};
    ($buffer:expr, index $i_op:tt $index:expr, offset $o_op:tt $offset:expr) => {{
        cursor!($buffer, index $i_op $index);
        cursor!($buffer, offset $o_op $offset);
    }};
}

impl Buffer {
    pub fn position(&self) -> usize {
        let (index, offset) = cursor!(&self);
        offset + self.text.line_to_byte(index)
    }

    pub const fn vscroll(&self) -> usize {
        self.vscroll
    }

    pub fn update_vscroll(&mut self, max: usize) {
        let index = cursor!(&self, index);
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
            position: Position::default(),
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
    use super::*;

    #[test]
    fn test_cursor_macro() {
        let mut buffer = Buffer::default();

        cursor!(buffer, index += 5);
        assert_eq!((5, 0), cursor!(buffer));

        cursor!(buffer, offset += 10);
        assert_eq!((5, 10), cursor!(buffer));

        cursor!(buffer, index = 15, offset = 20);
        assert_eq!((15, 20), cursor!(buffer));
    }
}
