use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use anyhow::Result;
use ropey::Rope;

#[derive(Default)]
pub struct Buffer {
    text: Rope,
    path: Option<PathBuf>,
    readonly: bool,
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

    pub fn save(&self) -> Result<()> {
        if let Some(path) = self.path.as_ref() {
            if !self.readonly {
                self.text.write_to(File::create(path)?)?;
            }
        }

        Ok(())
    }
}
