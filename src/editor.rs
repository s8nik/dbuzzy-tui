use std::{collections::HashMap, path::Path};

use anyhow::Result;

use crate::{
    buffer::{Buffer, BufferId},
    command::CommandExecutor,
    input::Input,
    keymap::Keymaps,
    widget::EditorWidget,
};

pub struct Editor<'a> {
    buffers: HashMap<BufferId, Buffer>,
    keymaps: &'static Keymaps,
    current: BufferId,
    viewport: (usize, usize),
    executor: CommandExecutor<'a>,
}

impl<'a> Editor<'a> {
    pub fn init() -> Self {
        Self {
            keymaps: Keymaps::init(),
            buffers: HashMap::new(),
            current: BufferId::MAX,
            viewport: (0, 0),
            executor: CommandExecutor::default(),
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

    pub fn viewport(&self) -> (usize, usize) {
        self.viewport
    }

    pub fn exit(&self) -> bool {
        self.executor.exit
    }

    pub fn widget(&self) -> EditorWidget {
        EditorWidget::new(self)
    }

    pub fn handle_event(&mut self, input: Input) {
        let buffer = self.buffers.get_mut(&self.current).expect("should exist");
        let cursor = &buffer.content().cursor;

        let bindings = self
            .keymaps
            .get(&cursor.mode)
            .expect("keymap must be registered");

        self.executor
            .execute(input, buffer, bindings, self.viewport.1);
    }

    pub fn set_viewport(&mut self, width: u16, height: u16) {
        self.viewport = (width as usize, height as usize);
    }
}
