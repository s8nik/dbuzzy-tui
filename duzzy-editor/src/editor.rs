use std::{collections::HashMap, path::Path};

use crate::{
    command::{insert_mode, CommandFinder},
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
        self.workspace.add_doc(doc);
        Ok(())
    }

    pub fn open_scratch(&mut self) {
        self.workspace.add_doc(Document::default());
    }

    pub const fn widget(&self) -> Renderer<'_> {
        Renderer::new(self)
    }

    pub const fn viewport(&self) -> (usize, usize) {
        (self.viewport.width, self.viewport.height)
    }

    pub fn cursor(&self) -> Cursor {
        let buf = self.workspace.curr().buf();
        let mode = buf.mode();

        let (mut y, mut x) = &buf.pos();

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
            None if buf.is_insert() => insert_mode::on_key(&mut self.workspace, input),
            _ => EventOutcome::Ignore,
        };

        if matches!(outcome, EventOutcome::Render) {
            self.workspace
                .curr_mut()
                .buf_mut()
                .update_vscroll(self.viewport.height);
        }

        outcome
    }
}

pub struct Workspace {
    documents: HashMap<DocumentId, Document>,
    current: DocumentId,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            current: DocumentId::MAX,
        }
    }

    pub(super) fn add_doc(&mut self, doc: Document) {
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
}
