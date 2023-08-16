use std::{collections::HashMap, path::Path};

use anyhow::Result;

use crate::{
    buffer::{Buffer, BufferId},
    command::Command,
    event::Input,
    keymap::Keymaps,
    mode::CursorMode,
    widget::EditorWidget,
};

pub struct Editor {
    buffers: HashMap<BufferId, Buffer>,
    keymaps: &'static Keymaps,
    current: BufferId,
    viewport: (usize, usize),
    // command: Command<'a>,
}

impl Editor {
    pub fn init() -> Self {
        Self {
            keymaps: Keymaps::init(),
            buffers: HashMap::new(),
            // command: Command::default(),
            current: BufferId::MAX,
            viewport: (0, 0),
        }
    }

    pub fn current_buff(&self) -> &Buffer {
        self.buffers.get(&self.current).expect("should exist")
    }

    pub fn current_buff_mut(&mut self) -> &mut Buffer {
        self.buffers.get_mut(&self.current).expect("should exist")
    }

    pub fn empty(&self) -> bool {
        self.current == BufferId::MAX
    }

    pub fn open(&mut self, filepath: impl AsRef<Path>) -> Result<()> {
        let buffer = Buffer::from_path(filepath)?;

        self.add_buffer(buffer);

        Ok(())
    }

    pub fn open_scratch(&mut self) {
        let buffer = Buffer::default();

        self.add_buffer(buffer);
    }

    fn add_buffer(&mut self, buffer: Buffer) {
        let buffer_id = buffer.id();
        self.buffers.insert(buffer_id, buffer);
        self.current = buffer_id;
    }

    // pub fn command(&self) -> &Command {
    //     &self.command
    // }

    pub fn viewport(&self) -> (usize, usize) {
        self.viewport
    }

    pub fn widget(&self) -> EditorWidget {
        EditorWidget::new(self)
    }

    pub fn cursor(&self) -> (usize, usize) {
        let buffer = self.current_buff();
        (
            buffer.cursor_offset(),
            buffer.line_index() - buffer.vscroll_index(),
        )
    }

    pub fn handle_event(&mut self, input: Input) {
        let current_mode = self.current_buff().cursor_mode();
        let buffer = self.buffers.get_mut(&self.current).expect("should exist");

        // let keymap = self
        //     .keymaps
        //     .get(current_mode)
        //     .expect("keymap should be registered!");

        // self.command.execute(input, buffer, keymap);

        // Command::scroll(buffer, self.viewport.1);
    }

    pub fn set_viewport(&mut self, width: u16, height: u16) {
        self.viewport = (width as usize, height as usize);
    }
}
