use std::{collections::HashMap, path::Path};

use crate::{
    command::{insert, CommandFinder},
    document::{Document, DocumentId},
    keymap::Keymaps,
    renderer::{Cursor, EventOutcome, Renderer, Viewport},
};

pub struct DuzzyEditor {
    pub(super) workspace: Workspace,
    keymaps: &'static Keymaps,
    command: CommandFinder,
    viewport: Viewport,
}

impl DuzzyEditor {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            workspace: Workspace::new(),
            keymaps: Keymaps::init(),
            command: CommandFinder::default(),
            viewport: Viewport { width, height },
        }
    }

    pub fn open_file(&mut self, filepath: impl AsRef<Path>) -> anyhow::Result<()> {
        let doc = Document::from_path(filepath)?;
        Ok(self.workspace.add_doc(doc))
    }

    pub fn open_scratch(&mut self) {
        self.workspace.add_doc(Document::default())
    }

    pub const fn widget(&self) -> Renderer<'_> {
        Renderer::new(&self)
    }

    pub const fn viewport(&self) -> (usize, usize) {
        (self.viewport.width, self.viewport.height)
    }

    pub fn cursor(&self) -> Cursor {
        let buf = self.workspace.curr().buf();
        let mode = buf.mode;

        let (mut y, mut x) = Into::into(&buf.pos);

        x = x.min(self.viewport.width - 1);
        y = y
            .saturating_sub(buf.vscroll())
            .min(self.viewport.height - 1);

        Cursor {
            x: x as _,
            y: y as _,
            mode,
        }
    }

    pub fn on_event(&mut self, event: crossterm::event::Event) -> EventOutcome {
        if let crossterm::event::Event::Resize(width, height) = event {
            self.viewport.update(width as _, height as _);
            return EventOutcome::Render;
        }

        let crossterm::event::Event::Key(e) = event else {
            return EventOutcome::Ignore;
        };

        let input = e.into();
        let buf = self.workspace.curr().buf();
        let command = self.command.find(self.keymaps, buf, input);

        let outcome = match command {
            Some(command) => {
                command.call(&mut self.workspace);
                self.command.reset();
                EventOutcome::Render
            }
            None if buf.is_insert() => insert::on_key(&mut self.workspace, input),
            _ => EventOutcome::Ignore,
        };

        if matches!(outcome, EventOutcome::Render) {
            let height = self.viewport.height;
            self.workspace.with_curr_mut(|doc| {
                doc.buf_mut().update_vscroll(height);
            });
        }

        outcome
    }
}

pub struct Workspace {
    documents: HashMap<DocumentId, Document>,
    current: DocumentId,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            current: DocumentId::MAX,
        }
    }

    fn add_doc(&mut self, doc: Document) {
        let id = doc.id();
        self.documents.insert(id, doc);
        self.current = id;
    }

    pub fn curr(&self) -> &Document {
        self.documents.get(&self.current).expect("current doc")
    }

    pub fn curr_mut(&mut self) -> &mut Document {
        self.documents
            .get_mut(&self.current)
            .expect("current mut doc")
    }

    pub fn with_curr<T, F>(&self, func: F) -> T
    where
        F: Fn(&Document) -> T,
    {
        let curr = self.curr();
        func(curr)
    }

    pub fn with_curr_mut<T, F>(&mut self, func: F) -> T
    where
        F: Fn(&mut Document) -> T,
    {
        let curr = self.curr_mut();
        func(curr)
    }
}
