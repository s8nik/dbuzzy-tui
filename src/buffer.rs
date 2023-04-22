use std::{
    fs::File,
    io::BufReader,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::Result;
use ropey::Rope;

use crate::mode::CursorMode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BufferId(NonZeroUsize);

impl BufferId {
    pub const MAX: Self = Self(unsafe { NonZeroUsize::new_unchecked(usize::MAX) });

    pub fn next() -> Self {
        pub static IDS: AtomicUsize = AtomicUsize::new(1);

        let next = NonZeroUsize::new(IDS.fetch_add(1, Ordering::SeqCst))
            .expect("BufferId counter overflowed");

        Self(next)
    }
}

impl Default for BufferId {
    fn default() -> Self {
        BufferId::next()
    }
}

#[derive(Default)]
pub struct Buffer {
    id: BufferId,
    text: Rope,
    path: Option<PathBuf>,
    readonly: bool,
    cursor_offset: usize,
    line_index: usize,
    cursor_mode: CursorMode,
}

impl Buffer {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut buffer = Self::default();

        if !path.exists() {
            buffer.path = Some(path.into());
            return Ok(buffer);
        }

        let metadata = path.metadata()?;
        if !metadata.is_file() {
            anyhow::bail!("Not a file: {}", path.display());
        }

        let file = File::open(path)?;
        let text = Rope::from_reader(BufReader::new(file))?;

        buffer.text = text;
        buffer.path = Some(path.into());
        buffer.readonly = metadata.permissions().readonly();

        Ok(buffer)
    }

    fn cursor_bound(&mut self) {
        let line_bytes_len = self.text.line(self.line_index).len_bytes();
        if self.cursor_offset > line_bytes_len {
            self.cursor_offset = line_bytes_len;
        }
    }

    pub fn id(&self) -> BufferId {
        self.id
    }

    pub const fn text(&self) -> &Rope {
        &self.text
    }

    pub fn text_mut(&mut self) -> &mut Rope {
        &mut self.text
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_ref().map(|m| m.as_ref())
    }

    pub fn readonly(&self) -> bool {
        self.readonly
    }

    pub fn cursor_offset(&self) -> usize {
        self.cursor_offset
    }

    pub fn line_index(&self) -> usize {
        self.line_index
    }

    pub fn move_back_by(&mut self, offset: usize) {
        self.cursor_offset = self.cursor_offset.saturating_sub(offset);
    }

    pub fn move_forward_by(&mut self, offset: usize) {
        let line_bytes_len = self.text.line(self.line_index).len_bytes();
        if (self.cursor_offset + offset) <= line_bytes_len {
            self.cursor_offset += offset
        }
    }

    pub fn move_up_by(&mut self, offset: usize) {
        self.line_index = self.line_index.saturating_sub(offset);
        self.cursor_bound();
    }

    pub fn move_down_by(&mut self, offset: usize) {
        if (self.line_index + offset) < self.text.len_lines() {
            self.line_index += offset;
        }
        self.cursor_bound();
    }

    pub fn cursor_position(&self) -> usize {
        let line_index = self.text.line_to_byte(self.line_index);
        line_index + self.cursor_offset
    }

    pub fn insert_char(&mut self, ch: char) {
        self.text.insert_char(self.cursor_position(), ch);
    }

    pub fn new_line(&mut self) {
        self.insert_char('\n');
        self.cursor_offset = 0;
        self.line_index += 1;
    }

    pub fn backspace(&mut self) {
        let pos = self.cursor_position();

        if pos != 0 {
            if self.cursor_offset == 0 {
                self.move_up_by(1);
                self.cursor_offset = self.text.line(self.line_index).len_bytes();
            }

            self.text.remove(pos - 1..pos);
            self.move_back_by(1);
        }
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = self.path.as_ref() {
            if !self.readonly {
                self.text.write_to(File::create(path)?)?;
            }
        }

        Ok(())
    }

    pub fn cursor_mode(&self) -> CursorMode {
        self.cursor_mode
    }

    pub fn set_cursor_mode(&mut self, mode: CursorMode) {
        self.cursor_mode = mode;
    }
}
