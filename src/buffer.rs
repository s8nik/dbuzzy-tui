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

    pub fn set_cursor_offset(&mut self, new_offset: usize) {
        self.cursor_offset = new_offset;
    }

    pub fn line_index(&self) -> usize {
        self.line_index
    }

    pub fn set_line_index(&mut self, new_index: usize) {
        self.line_index = new_index;
    }

    pub fn cursor_position(&self) -> usize {
        let line_index = self.text.line_to_byte(self.line_index);
        line_index + self.cursor_offset
    }

    pub fn cursor_mode(&self) -> CursorMode {
        self.cursor_mode
    }

    pub fn set_cursor_mode(&mut self, mode: CursorMode) {
        self.cursor_mode = mode;
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = self.path.as_ref() {
            if !self.readonly {
                self.text.write_to(File::create(path)?)?;
            }
        }

        Ok(())
    }
}
