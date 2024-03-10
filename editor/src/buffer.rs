use std::{
    fs::File,
    io::BufReader,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::Result;
use ropey::Rope;

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

    pub text: Rope,
    pub offset: usize,
    pub index: usize,
    pub vscroll: usize,
    pub mode: CursorMode,
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

    pub fn position(&self) -> usize {
        let byte_index = self.text.line_to_byte(self.index);
        self.offset + byte_index
    }

    pub fn update_vscroll(&mut self, max: usize) {
        let upper_bound = self.vscroll + max - 1;

        if self.index < self.vscroll {
            self.vscroll = self.index;
        } else if self.index > upper_bound {
            self.vscroll = self.index - max + 1;
        }
    }

    pub fn line_len_bytes(&self) -> usize {
        self.text.line(self.index).len_bytes()
    }

    pub fn len_lines(&self) -> usize {
        self.text.len_lines()
    }

    pub fn is_insert(&self) -> bool {
        self.mode == CursorMode::Insert
    }

    pub fn is_normal(&self) -> bool {
        self.mode == CursorMode::Normal
    }
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum CursorMode {
    Insert,
    #[default]
    Normal,
    Visual,
}
