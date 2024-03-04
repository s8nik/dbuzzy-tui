use std::path::Path;

use crate::{
    buffer::{Buffer, BufferId},
    commands::CommandExecutor,
    keymap::Keymaps,
    widget::EditorWidget,
    workspace::Workspace,
};

pub struct Editor<'a> {
    pub workspace: Workspace,
    pub keymaps: &'static Keymaps,
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
            workspace.current = buffer_id;
        });

        Ok(())
    }

    pub fn open_scratch(&mut self) {
        let buffer = Buffer::default();
        self.open_impl(buffer, |workspace, buffer_id| {
            workspace.current = buffer_id;
        });
    }

    pub fn open_logger(&mut self) {
        let buffer = Buffer::logger();
        self.open_impl(buffer, |workspace, buffer_id| {
            workspace.logger = buffer_id;
        });
    }

    fn open_impl(&mut self, buffer: Buffer, set_buff_id: fn(&mut Workspace, BufferId)) {
        let buffer_id = self.workspace.add_buffer(buffer);
        set_buff_id(&mut self.workspace, buffer_id)
    }

    pub fn widget(&self) -> EditorWidget {
        EditorWidget::new(self)
    }

    pub fn on_event(&mut self, event: crossterm::event::Event) -> bool {
        if let crossterm::event::Event::Resize(w, h) = event {
            self.workspace.viewport.update(w as usize, h as usize);
            return true;
        }

        let crossterm::event::Event::Key(e) = event else {
            return false;
        };

        let input = e.into();

        let Some(buffer) = current!(self.workspace) else {
            log::error!("no active buffer");
            return false;
        };

        let bindings = self
            .keymaps
            .get(&buffer.cursor_mode())
            .expect("keymap must be registered");

        // @todo: return an enum
        // @todo: update vscroll
        self.executor.execute(input, &mut self.workspace, bindings)
    }

    pub fn on_log(&mut self, log: ropey::Rope) -> bool {
        if let Some(log_buffer) = current_mut!(self.workspace, logger) {
            log_buffer.text_mut().append(log);
        }

        self.workspace.is_current_logger()
    }
}
