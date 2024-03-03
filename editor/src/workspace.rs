use std::collections::HashMap;

use crate::buffer::{Buffer, BufferId};

pub struct Workspace {
    buffers: HashMap<BufferId, Buffer>,
    logger: BufferId,
    current: BufferId,
    viewport: Viewport,
}

impl Workspace {
    pub fn init(width: usize, heigth: usize) -> Self {
        Self {
            current: BufferId::MAX,
            logger: BufferId::MAX,
            buffers: HashMap::new(),
            viewport: Viewport {
                x: width,
                y: heigth,
            },
        }
    }

    pub fn current_buff(&self) -> Option<&Buffer> {
        self.buffers.get(&self.current)
    }

    pub fn current_buff_mut(&mut self) -> Option<&mut Buffer> {
        self.buffers.get_mut(&self.current)
    }

    pub fn empty(&self) -> bool {
        self.current == BufferId::MAX
    }

    pub fn viewport(&self) -> &Viewport {
        &self.viewport
    }

    pub fn add_buffer(&mut self, buffer: Buffer) -> BufferId {
        let buffer_id = buffer.id();
        self.buffers.insert(buffer_id, buffer);
        buffer_id
    }

    pub fn set_current(&mut self, buffer_id: BufferId) {
        self.current = buffer_id;
    }

    pub fn set_logger(&mut self, buffer_id: BufferId) {
        self.logger = buffer_id;
    }

    pub fn set_viewport(&mut self, x: usize, y: usize) {
        self.viewport.update(x, y)
    }
}

#[derive(Default)]
pub struct Viewport {
    pub x: usize,
    pub y: usize,
}

impl Viewport {
    pub fn update(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }
}
