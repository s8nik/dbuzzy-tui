use std::path::Path;

use crate::{
    buffer::{Buffer, BufferId},
    command::CommandExecutor,
    keymap::Keymaps,
    widget::EditorWidget,
    workspace::Workspace,
};

pub struct Editor<'a> {
    workspace: Workspace,
    keymaps: &'static Keymaps,
    executor: CommandExecutor<'a>,
}

impl<'a> Editor<'a> {
    pub fn init(width: usize, height: usize) -> Self {
        Self {
            workspace: Workspace::init(width, height),
            keymaps: Keymaps::init(),
            executor: CommandExecutor::default(),
        }
    }

    pub fn open(&mut self, filepath: impl AsRef<Path>) -> anyhow::Result<()> {
        let buffer = Buffer::from_path(filepath)?;
        self.open_impl(buffer, |workspace, buffer_id| {
            workspace.set_current(buffer_id)
        });

        Ok(())
    }

    pub fn open_scratch(&mut self) {
        let buffer = Buffer::default();
        self.open_impl(buffer, |workspace, buffer_id| {
            workspace.set_current(buffer_id)
        });
    }

    pub fn open_logger(&mut self) {
        let buffer = Buffer::logger();
        self.open_impl(buffer, |workspace, buffer_id| {
            workspace.set_logger(buffer_id)
        });
    }

    fn open_impl(&mut self, buffer: Buffer, set_buff_id: fn(&mut Workspace, BufferId)) {
        let buffer_id = self.workspace.add_buffer(buffer);
        set_buff_id(&mut self.workspace, buffer_id)
    }

    pub fn workspace(&self) -> &Workspace {
        &self.workspace
    }

    pub fn widget(&self) -> EditorWidget {
        EditorWidget::new(&self)
    }

    // pub fn on_event(&mut self, event: crossterm::event::Event) -> bool {
    //     if let crossterm::event::Event::Resize(w, h) = event {
    //         self.set_viewport(w, h);
    //         return true;
    //     }

    //     let crossterm::event::Event::Key(e) = event else {
    //         return false;
    //     };

    //     let input = e.into();
    //     let buffer = self.buffers.get_mut(&self.current).expect("should exist");

    //     let bindings = self
    //         .keymaps
    //         .get(&buffer.content().cursor.mode)
    //         .expect("keymap must be registered");

    //     let content = buffer.content_mut();

    //     let mut consumed = false;

    //     // @todo: refactor
    //     if content.cursor.mode == CursorMode::Insert {
    //         match input {
    //             Input {
    //                 event: crate::input::Event::Char('q'),
    //                 modifiers: crate::input::Modifiers { ctr: true, .. },
    //             } => self.exit = true,
    //             Input {
    //                 event: crate::input::Event::Char(ch),
    //                 modifiers: _,
    //             } => {
    //                 CommandExecutor::enter(content, ch);
    //                 consumed = true;
    //             }
    //             _ => (),
    //         }
    //     }

    //     let executed = self.executor.execute(input, content, bindings);
    //     content.cursor.scroll(self.viewport.1);

    //     consumed || executed
    // }

    // pub fn on_log(&mut self, log: ropey::Rope) -> bool {
    //     if let Some(log_buffer) = self.buffers.get_mut(&self.logger) {
    //         log_buffer.content_mut().text.append(log);
    //     }

    //     self.current == self.logger
    // }
}
