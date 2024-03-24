use std::{
    fs::File,
    io::BufReader,
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use ropey::Rope;

use crate::{
    buffer::Buffer,
    history::History,
    transaction::{Transaction, TransactionResult},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentId(NonZeroUsize);

impl DocumentId {
    pub const MAX: Self = Self(unsafe { NonZeroUsize::new_unchecked(usize::MAX) });

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
    transaction: Option<Transaction>,
}

impl Document {
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

        document.buffer.set_text(text);
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

    pub fn with_transaction<F>(&mut self, func: F)
    where
        F: Fn(&mut Transaction, &mut Buffer) -> TransactionResult,
    {
        let mut tx = match self.transaction.take() {
            Some(transaction) => transaction,
            None => Transaction::new(),
        };

        match func(&mut tx, &mut self.buffer) {
            TransactionResult::Commit => self.history.commit(tx),
            TransactionResult::Keep => self.transaction = Some(tx),
            TransactionResult::Abort => (),
        }
    }

    pub fn commit(&mut self) {
        if let Some(tx) = self.transaction.take() {
            self.history.commit(tx);
        }
    }

    pub fn undo(&mut self) -> Option<usize> {
        self.history.undo(self.buffer.text_mut())
    }

    pub fn redo(&mut self) -> Option<usize> {
        self.history.redo(self.buffer.text_mut())
    }
}
