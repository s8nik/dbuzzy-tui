use std::{
    fs::File,
    io::BufReader,
    num::NonZeroUsize,
    path::Path,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::Result;
use ropey::Rope;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferId(NonZeroUsize);

impl BufferId {
    pub fn next() -> Self {
        pub static IDS: AtomicUsize = AtomicUsize::new(1);

        let next = NonZeroUsize::new(IDS.fetch_add(1, Ordering::SeqCst))
            .expect("BufferId counter overflowed");

        Self(next)
    }
}

impl Default for BufferId {
    fn default() -> Self {
        Self::next()
    }
}

#[derive(Debug, Default)]
pub struct FileMeta {
    pub path: Option<PathBuf>,
    pub readonly: bool,
}

#[derive(Debug, Default)]
pub struct Position {
    pub index: usize,
    pub offset: usize,
}

#[derive(Debug)]
pub struct Buffer {
    id: BufferId,
    meta: FileMeta,

    pub(super) text: Rope,
    pub(super) position: Position,
    vscroll: usize,

    mode: CursorMode,
    available_modes: Vec<CursorMode>,
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
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut buffer = Self::default();

        if !path.exists() {
            buffer.meta.path = Some(path.into());
            return Ok(buffer);
        }

        let metadata = path.metadata()?;
        if !metadata.is_file() {
            anyhow::bail!("Not a file: {}", path.display());
        }

        let file = File::open(path)?;
        let text = Rope::from_reader(BufReader::new(file))?;

        buffer.text = text;
        buffer.meta = FileMeta {
            path: Some(path.into()),
            readonly: metadata.permissions().readonly(),
        };

        Ok(buffer)
    }

    pub fn logger() -> Self {
        Self {
            id: BufferId::next(),
            meta: FileMeta {
                path: None,
                readonly: true,
            },
            text: Rope::default(),
            position: Position::default(),
            vscroll: 0,
            mode: CursorMode::Normal,
            available_modes: vec![CursorMode::Normal, CursorMode::Visual],
        }
    }

    pub const fn id(&self) -> BufferId {
        self.id
    }

    pub const fn cursor_mode(&self) -> CursorMode {
        self.mode
    }

    pub fn update_cursor_mode(&mut self, mode: CursorMode) {
        if self.available_modes.contains(&mode) {
            self.mode = mode;
        }
    }

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
            id: BufferId::default(),
            meta: FileMeta::default(),
            text: Rope::default(),
            position: Position::default(),
            vscroll: 0,
            mode: CursorMode::Normal,
            available_modes: vec![CursorMode::Insert, CursorMode::Normal, CursorMode::Visual],
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
