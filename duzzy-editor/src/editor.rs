use std::{cell::RefCell, collections::HashMap, path::Path};

use duzzy_lib::{event::Input, EventOutcome, OnInput};

use crate::{
    clipboard::Clipboard,
    command::{input_on_key, search_on_key, CommandFinder},
    document::{Document, DocumentId},
    keymap::Keymaps,
    search::SearchRegistry,
    widget::{Cursor, EditorWidget, Viewport},
    SmartString,
};

pub struct Editor {
    pub(super) workspace: Workspace,
    pub(super) viewport: RefCell<Viewport>,

    keymaps: &'static Keymaps,
    command: CommandFinder,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    pub fn new() -> Self {
        Self {
            workspace: Workspace::new(),
            keymaps: Keymaps::init(),
            command: CommandFinder::default(),
            viewport: RefCell::new(Viewport::default()),
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

    pub fn widget(&self) -> EditorWidget<'_> {
        EditorWidget::new(self)
    }

    pub fn cursor(&self) -> Cursor {
        let buf = self.workspace.cur().buf();
        let mode = buf.mode();

        let viewport = self.viewport.borrow();
        let (mut y, mut x) = buf.pos();

        x = x.min(viewport.width - 1);
        y = y.saturating_sub(buf.vscroll()).min(viewport.height - 1);

        Cursor {
            x: x as _,
            y: y as _,
            mode,
        }
    }
}

impl OnInput for Editor {
    fn on_input(&mut self, input: Input) -> EventOutcome {
        let buf = self.workspace.cur().buf();
        let command = self.command.find(self.keymaps, buf, input);

        let outcome = match command {
            Some(command) => {
                command.call(&mut self.workspace);
                self.command.reset();
                EventOutcome::Render
            }
            None if buf.is_insert() => input_on_key(&mut self.workspace, input),
            None if buf.is_search() => search_on_key(&mut self.workspace, input),
            _ => EventOutcome::Ignore,
        };

        if matches!(outcome, EventOutcome::Render) {
            let viewport = self.viewport.borrow();
            self.workspace
                .cur_mut()
                .buf_mut()
                .update_vscroll(viewport.height);
        }

        outcome
    }
}

pub struct Workspace {
    documents: HashMap<DocumentId, Document>,
    current: DocumentId,
    clipboard: Clipboard,
    search_registry: SearchRegistry,
    pub(super) search_buffer: SmartString,
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            current: DocumentId::MAX,
            documents: HashMap::new(),
            clipboard: Clipboard::new(),
            search_buffer: SmartString::new_const(),
            search_registry: SearchRegistry::default(),
        }
    }

    pub(super) fn add_doc(&mut self, doc: Document) {
        let id = doc.id();
        self.documents.insert(id, doc);
        self.current = id;
    }

    pub fn clipboard(&mut self) -> &mut Clipboard {
        &mut self.clipboard
    }

    pub fn apply_search(&mut self) {
        if !self.search_buffer.is_empty() {
            let pattern = std::mem::take(&mut self.search_buffer);
            self.search_registry = SearchRegistry::new(pattern);
        }
    }

    pub const fn search_registry(&self) -> &SearchRegistry {
        &self.search_registry
    }

    pub fn cur(&self) -> &Document {
        self.documents.get(&self.current).expect("current doc")
    }

    pub fn cur_mut(&mut self) -> &mut Document {
        self.documents
            .get_mut(&self.current)
            .expect("current mut doc")
    }
}
