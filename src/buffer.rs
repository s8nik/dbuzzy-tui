use std::{
    fs::File,
    io::BufReader,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::Result;
use ropey::Rope;

use crate::cursor::Cursor;

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
pub struct Content {
    pub text: Rope,
    pub cursor: Cursor,
}

#[derive(Default)]
pub struct Buffer {
    id: BufferId,
    meta: FileMeta,
    content: Content,
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

        buffer.content.text = text;
        buffer.meta = FileMeta {
            path: Some(path.into()),
            readonly: metadata.permissions().readonly(),
        };

        Ok(buffer)
    }

    pub fn id(&self) -> BufferId {
        self.id
    }

    pub fn content(&self) -> &Content {
        &self.content
    }

    pub fn content_mut(&mut self) -> &mut Content {
        &mut self.content
    }

    // pub fn save(&self) -> Result<()> {
    //     let FileMeta { path, readonly } = &self.meta;

    //     if let Some(path) = path.as_ref() {
    //         if !readonly {
    //             self.content.text.write_to(File::create(path)?)?;
    //         }
    //     }

    //     Ok(())
    // }
}
