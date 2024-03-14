use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use ropey::Rope;

use crate::{buffer::Buffer, history::History};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentId(NonZeroUsize);

impl DocumentId {
    pub fn next() -> Self {
        pub static IDS: AtomicUsize = AtomicUsize::new(1);

        let next = NonZeroUsize::new(IDS.fetch_add(1, Ordering::SeqCst))
            .expect("Document id counter overflowed");

        Self(next)
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::next()
    }
}

#[derive(Debug, Default)]
pub struct FileMeta {
    pub path: Option<PathBuf>,
    pub readonly: bool,
}

#[derive(Default)]
pub struct Document {
    id: DocumentId,
    meta: FileMeta,
    buffer: Buffer,
    history: History,
}

impl Document {
    pub fn logger() -> Self {
        Self {
            id: DocumentId::next(),
            buffer: Buffer::logger(),
            history: History::default(),
            meta: FileMeta {
                path: None,
                readonly: true,
            },
        }
    }

    pub fn from_path(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let mut document = Self::default();

        if !path.exists() {
            document.meta.path = Some(path.into());
            return Ok(document);
        }

        let metadata = path.metadata()?;
        if !metadata.is_file() {
            anyhow::bail!("Not a file: {}", path.display());
        }

        let file = File::open(path)?;
        let text = Rope::from_reader(BufReader::new(file))?;

        document.buffer.text = text;
        document.meta = FileMeta {
            path: Some(path.into()),
            readonly: metadata.permissions().readonly(),
        };

        Ok(document)
    }

    pub const fn id(&self) -> DocumentId {
        self.id
    }

    pub const fn buf(&self) -> &Buffer {
        &self.buffer
    }

    pub fn buf_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }
}

#[derive(Default)]
pub struct Workspace {
    documents: HashMap<DocumentId, Document>,
    current: DocumentId,
    logger: DocumentId,
}

impl Workspace {
    pub fn current(&self) -> &Document {
        self.documents.get(&self.current).expect("current buff")
    }

    pub fn current_mut(&mut self) -> &mut Document {
        self.documents
            .get_mut(&self.current)
            .expect("current mut buff")
    }

    pub fn logger(&mut self) -> Option<&mut Document> {
        self.documents.get_mut(&self.logger)
    }

    pub fn add_document(&mut self, document: Document, set_current: bool) {
        let id = document.id();
        self.documents.insert(id, document);

        if set_current {
            self.current = id;
        }
    }

    pub fn init_logger(&self) {
        let document = Document::logger();
        let id = document.id();

        self.documents.insert(id, document);
        self.logger = id;
    }

    pub fn logger_active(&self) -> bool {
        self.current == self.logger
    }
}
