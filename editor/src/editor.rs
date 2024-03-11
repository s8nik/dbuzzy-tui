use std::path::Path;

use crate::{
    add_buffer,
    buffer::Buffer,
    command::{insert, CommandFinder},
    keymap::Keymaps,
    renderer::{Cursor, EventOutcome, Renderer, Viewport},
    workspace::Workspace,
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
        add_buffer!(workspace, Buffer::logger(), logger);

        Self {
            workspace,
            keymaps: Keymaps::init(),
            command: CommandFinder::default(),
            viewport: Viewport { width, height },
        }
    }

    pub fn open_file(&mut self, filepath: impl AsRef<Path>) -> anyhow::Result<()> {
        let buffer = Buffer::from_path(filepath)?;
        add_buffer!(self.workspace, buffer, current);

        Ok(())
    }

    pub fn open_scratch(&mut self) {
        add_buffer!(self.workspace, Buffer::default(), current);
    }

    pub const fn widget(&self) -> Renderer<'_> {
        Renderer::new(self)
    }

    pub fn cursor(&self) -> Cursor {
        let buffer = self.workspace.current();
        let mode = buffer.cursor_mode();

        let mut x = buffer.offset;
        let mut y = buffer.index;

        x = x.min(self.viewport.width - 1);
        y = y
            .saturating_sub(buffer.vscroll)
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

        let is_insert = self.workspace.current().is_insert();

        let input = e.into();
        let command = self.command.find(self.keymaps, &self.workspace, input);

        let outcome = match command {
            Some(command) => {
                command.call(&mut self.workspace);
                self.command.reset();
                EventOutcome::Render
            }
            None if is_insert => insert::on_key(self.workspace.current_mut(), input),
            _ => EventOutcome::Ignore,
        };

        if let EventOutcome::Render = outcome {
            self.workspace
                .current_mut()
                .update_vscroll(self.viewport.height);
        }

        outcome
    }

    pub fn on_log(&mut self, log: ropey::Rope) -> EventOutcome {
        if let Some(buffer) = self.workspace.logger() {
            buffer.text.append(log);
        }

        match self.workspace.logger_active() {
            true => EventOutcome::Render,
            false => EventOutcome::Ignore,
        }
    }
}
