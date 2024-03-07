use std::path::Path;

use crate::{
    add_buffer,
    buffer::Buffer,
    command::CommandRegistry,
    keymap::Keymaps,
    renderer::{Cursor, EventOutcome, Renderer, Viewport},
    workspace::Workspace,
};

pub struct Editor {
    pub(crate) workspace: Workspace,
    pub(crate) viewport: Viewport,
    keymaps: &'static Keymaps,
    registry: CommandRegistry,
}

impl Editor {
    pub fn init(width: usize, height: usize) -> Self {
        let mut workspace = Workspace::default();

        // init logger
        add_buffer!(workspace, Buffer::logger(), logger);

        Self {
            workspace,
            keymaps: Keymaps::init(),
            registry: CommandRegistry::register(),
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

        let index = buffer.index();
        let vscroll = buffer.vscroll();

        let x = self.viewport.width.min(index.saturating_sub(vscroll)) as u16;
        let y = buffer.index() as u16;

        Cursor { x, y, mode }
    }

    // @todo:
    // 0. finish find/exec command & logs
    // 1. widget -> render
    // 2. refactoring of keymap (register keymaps in the file)
    // 3. clean up in command
    // pub fn on_event(&mut self, event: crossterm::event::Event) -> bool {
    //     if let crossterm::event::Event::Resize(w, h) = event {
    //         self.viewport.update(w as usize, h as usize);
    //         return true;
    //     }

    //     let crossterm::event::Event::Key(e) = event else {
    //         return false;
    //     };

    //     let input = e.into();

    //     let Some(buffer) = self.current() else {
    //         log::error!("no active buffer");
    //         return false;
    //     };

    //     let bindings = self
    //         .keymaps
    //         .get(&buffer.cursor_mode())
    //         .expect("keymap must be registered");

    //     // @todo: return an enum
    //     // @todo: update vscroll
    //     self.executor.execute(input, &mut self.workspace, bindings)
    // }

    pub fn on_event(&mut self, event: crossterm::event::Event) -> EventOutcome {
        if let crossterm::event::Event::Resize(width, height) = event {
            self.viewport.update(width as _, height as _);
            return EventOutcome::Render(true);
        }

        let crossterm::event::Event::Key(e) = event else {
            return EventOutcome::Render(false);
        };

        // let input = e.into();

        EventOutcome::Render(true)
    }

    pub fn on_log(&mut self, log: ropey::Rope) -> bool {
        if let Some(buffer) = self.workspace.logger() {
            buffer.text_mut().append(log);
        }

        self.workspace.logger_active()
    }
}
