use std::{
    fs::File,
    io::BufReader,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::Result;
use ropey::Rope;
use strum::EnumString;

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
pub struct FileMeta {
    pub path: Option<PathBuf>,
    pub readonly: bool,
}

#[derive(Default)]
pub struct Buffer {
    id: BufferId,
    meta: FileMeta,
    text: Rope,
    offset: usize,
    index: usize,
    vscroll: usize,
    mode: CursorMode,
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
            offset: 0,
            index: 0,
            vscroll: 0,
            mode: CursorMode::Normal,
        }
    }

    pub fn id(&self) -> BufferId {
        self.id
    }

    pub fn text(&self) -> &Rope {
        &self.text
    }

    pub fn text_mut(&mut self) -> &mut Rope {
        &mut self.text
    }

    pub fn position(&self) -> usize {
        let byte_index = self.text.line_to_byte(self.index);
        self.offset + byte_index
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    pub fn update_offset(&mut self, offset: usize) {
        self.offset = offset
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn update_index(&mut self, index: usize) {
        self.index = index;
    }

    pub fn vscroll(&self) -> usize {
        self.vscroll
    }

    pub fn update_vscroll(&mut self, max: usize) {
        let lower_bound = self.vscroll;
        let upper_bound = self.vscroll + max - 1;

        if self.index >= upper_bound {
            self.vscroll += self.index - upper_bound;
        } else if self.index < lower_bound {
            self.vscroll -= lower_bound - self.index;
        }
    }

    pub fn cursor_mode(&self) -> CursorMode {
        self.mode
    }

    pub fn update_cursor_mode(&mut self, mode: CursorMode) {
        self.mode = mode;
    }

    pub fn line_len_bytes(&self) -> usize {
        self.text.line(self.index).len_bytes()
    }

    pub fn len_lines(&self) -> usize {
        self.text.len_lines()
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum CursorMode {
    Insert,
    #[default]
    Normal,
    Visual,
}
