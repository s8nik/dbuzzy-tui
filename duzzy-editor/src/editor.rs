use std::{collections::HashMap, path::Path};

use duzzy_lib::{event::Input, DuzzyWidget, EventOutcome, NamedWidget};

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
    pub(super) viewport: Viewport,
    keymaps: &'static Keymaps,
    command: CommandFinder,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new_scratch()
    }
}

impl Editor {
    fn new(workspace: Workspace) -> Self {
        Self {
            workspace,
            viewport: Viewport::default(),
            keymaps: Keymaps::init(),
            command: CommandFinder::default(),
        }
    }

    pub fn new_file(filepath: impl AsRef<Path>) -> anyhow::Result<Self> {
        let mut workspace = Workspace::default();
        let doc = Document::from_path(filepath)?;
        workspace.add_doc(doc);
        Ok(Self::new(workspace))
    }

    pub fn new_scratch() -> Self {
        let mut workspace = Workspace::default();
        workspace.add_doc(Document::default());
        Self::new(workspace)
    }

    pub fn cursor(&self) -> Cursor {
        let buf = self.workspace.cur().buf();
        let mode = buf.mode();

        let (mut y, mut x) = buf.pos();
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
}

impl DuzzyWidget for Editor {
    fn input(&mut self, input: Input) -> EventOutcome {
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
            self.workspace
                .cur_mut()
                .buf_mut()
                .update_vscroll(self.viewport.height);
        }

        outcome
    }

    fn render(&mut self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        EditorWidget::new(self).render(area, buf);
    }
}

impl NamedWidget for Editor {
    fn name() -> &'static str {
        "duzzy-editor"
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
