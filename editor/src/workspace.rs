use std::{collections::HashMap, path::Path};

use anyhow::Result;

use crate::{
    buffer::{Buffer, BufferId},
    command::Executor,
    cursor::CursorMode,
    input::Input,
    keymap::Keymaps,
    widget::EditorWidget,
};

pub struct Workspace<'a> {
    buffers: HashMap<BufferId, Buffer>,
    keymaps: &'static Keymaps,
    logger: BufferId,
    current: BufferId,
    viewport: (usize, usize),
    executor: Executor<'a>,
    pub exit: bool,
}

impl<'a> Workspace<'a> {
    pub fn init() -> Self {
        Self {
            keymaps: Keymaps::init(),
            buffers: HashMap::new(),
            current: BufferId::MAX,
            logger: BufferId::MAX,
            viewport: (0, 0),
            executor: Executor::default(),
            exit: false,
        }
    }

    pub fn current_buff(&self) -> &Buffer {
        self.buffers.get(&self.current).expect("should exist")
    }

    pub fn empty(&self) -> bool {
        self.current == BufferId::MAX
    }

    pub fn open(&mut self, filepath: impl AsRef<Path>) -> Result<()> {
        let buffer = Buffer::from_path(filepath)?;
        self.current = self.add_buffer(buffer);

        Ok(())
    }

    pub fn open_scratch(&mut self) {
        let buffer = Buffer::default();
        self.current = self.add_buffer(buffer);
    }

    pub fn add_buffer(&mut self, buffer: Buffer) -> BufferId {
        let buffer_id = buffer.id();
        self.buffers.insert(buffer_id, buffer);
        buffer_id
    }

    pub fn set_logger(&mut self, buffer_id: BufferId) {
        self.logger = buffer_id;
    }

    pub fn viewport(&self) -> (usize, usize) {
        self.viewport
    }

    pub fn widget(&self) -> EditorWidget {
        EditorWidget::new(self)
    }

    pub fn set_viewport(&mut self, width: u16, height: u16) {
        self.viewport = (width as usize, height as usize);
    }

    pub fn on_event(&mut self, event: crossterm::event::Event) -> bool {
        if let crossterm::event::Event::Resize(w, h) = event {
            self.set_viewport(w, h);
            return true;
        }

        let crossterm::event::Event::Key(e) = event else {
            return false;
        };

        let input = e.into();
        let buffer = self.buffers.get_mut(&self.current).expect("should exist");

        let bindings = self
            .keymaps
            .get(&buffer.content().cursor.mode)
            .expect("keymap must be registered");

        let content = buffer.content_mut();

        let mut consumed = false;

        // @todo: refactor
        if content.cursor.mode == CursorMode::Insert {
            match input {
                Input {
                    event: crate::input::Event::Char('q'),
                    modifiers: crate::input::Modifiers { ctr: true, .. },
                } => self.exit = true,
                Input {
                    event: crate::input::Event::Char(ch),
                    modifiers: _,
                } => {
                    Executor::enter(content, ch);
                    consumed = true;
                }
                _ => (),
            }
        }

        let executed = self.executor.execute(input, content, bindings);
        content.cursor.scroll(self.viewport.1);

        consumed || executed
    }

    pub fn on_log(&mut self, log: ropey::Rope) -> bool {
        if let Some(log_buffer) = self.buffers.get_mut(&self.logger) {
            log_buffer.content_mut().text.append(log);
        }

        self.current == self.logger
    }
}
