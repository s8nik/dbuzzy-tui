use std::path::Path;

use crate::{
    add_buffer,
    buffer::{Buffer, CursorMode},
    command::{insert_mode_on_key, CommandResolver},
    keymap::Keymaps,
    renderer::{Cursor, EventOutcome, Renderer, Viewport},
    workspace::Workspace,
};

pub struct Editor {
    pub(crate) workspace: Workspace,
    pub(crate) viewport: Viewport,
    keymaps: &'static Keymaps,
    resolver: CommandResolver,
}

impl Editor {
    pub fn init(width: usize, height: usize) -> Self {
        let mut workspace = Workspace::default();

        // init logger
        add_buffer!(workspace, Buffer::logger(), logger);

        Self {
            workspace,
            keymaps: Keymaps::init(),
            resolver: CommandResolver::default(),
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

    pub fn widget(&self) -> Renderer<'_> {
        Renderer::new(self)
    }

    pub fn cursor(&self) -> Cursor {
        let buffer = self.workspace.current();
        let mode = buffer.cursor_mode();

        let mut x = buffer.offset();
        let mut y = buffer.index();

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
            return EventOutcome::Render(true);
        }

        let crossterm::event::Event::Key(e) = event else {
            return EventOutcome::Render(false);
        };

        let mode = self.workspace.current().cursor_mode();

        let input = e.into();
        let command = self.resolver.resolve(self.keymaps, &self.workspace, input);

        let outcome = if let Some(command) = command {
            command.call(&mut self.workspace);
            self.resolver.reset();

            EventOutcome::Render(true)
        } else if mode == CursorMode::Insert {
            insert_mode_on_key(self.workspace.current_mut(), input)
        } else {
            EventOutcome::Render(false)
        };

        if let EventOutcome::Render(true) = outcome {
            self.workspace
                .current_mut()
                .update_vscroll(self.viewport.height);
        }

        outcome
    }

    pub fn on_log(&mut self, log: ropey::Rope) -> EventOutcome {
        if let Some(buffer) = self.workspace.logger() {
            buffer.text_mut().append(log);
        }

        EventOutcome::Render(self.workspace.logger_active())
    }
}
