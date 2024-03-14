use std::path::Path;

use crate::{
    command::{insert, CommandFinder},
    cursor,
    keymap::Keymaps,
    renderer::{Cursor, EventOutcome, Renderer, Viewport},
    workspace::{Document, Workspace},
};

pub struct Editor {
    pub(crate) workspace: Workspace,
    pub(crate) viewport: Viewport,
    keymaps: &'static Keymaps,
    command: CommandFinder,
}

impl Editor {
    pub fn init(width: usize, height: usize) -> Self {
        let mut workspace = Workspace::default();

        // init logger
        // @note: remove it later
        workspace.init_logger();

        Self {
            workspace,
            keymaps: Keymaps::init(),
            command: CommandFinder::default(),
            viewport: Viewport { width, height },
        }
    }

    pub fn open_file(&mut self, filepath: impl AsRef<Path>) -> anyhow::Result<()> {
        let document = Document::from_path(filepath)?;
        self.workspace.add_document(document, true);

        Ok(())
    }

    pub fn open_scratch(&mut self) {
        let document = Document::default();
        self.workspace.add_document(document, true);
    }

    pub const fn widget(&self) -> Renderer<'_> {
        Renderer::new(self)
    }

    pub fn cursor(&self) -> Cursor {
        let buffer = self.workspace.current().buf();
        let mode = buffer.cursor_mode();

        let (mut y, mut x) = cursor!(buffer);

        x = x.min(self.viewport.width - 1);
        y = y
            .saturating_sub(buffer.vscroll())
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

        let buffer = self.workspace.current_mut().buf_mut();
        let is_insert = buffer.is_insert();

        let input = e.into();
        let command = self.command.find(self.keymaps, &self.workspace, input);

        let outcome = match command {
            Some(command) => {
                command.call(&mut self.workspace);
                self.command.reset();
                EventOutcome::Render
            }
            None if is_insert => insert::on_key(buffer, input),
            _ => EventOutcome::Ignore,
        };

        if let EventOutcome::Render = outcome {
            buffer.update_vscroll(self.viewport.height);
        }

        outcome
    }

    pub fn on_log(&mut self, log: ropey::Rope) -> EventOutcome {
        if let Some(doc) = self.workspace.logger() {
            doc.buf_mut().text.append(log);
        }

        match self.workspace.logger_active() {
            true => EventOutcome::Render,
            false => EventOutcome::Ignore,
        }
    }
}
