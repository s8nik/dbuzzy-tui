use std::collections::HashMap;

use crate::buffer::{Buffer, BufferId};

pub struct Workspace {
    pub buffers: HashMap<BufferId, Buffer>,
    pub current: BufferId,
    pub logger: BufferId,
    pub viewport: Viewport,
    pub exit: bool,
}

impl Workspace {
    pub fn init(width: usize, height: usize) -> Self {
        Self {
            current: BufferId::MAX,
            logger: BufferId::MAX,
            buffers: HashMap::new(),
            exit: false,
            viewport: Viewport {
                x: width,
                y: height,
            },
        }
    }

    pub fn empty(&self) -> bool {
        self.current == BufferId::MAX
    }

    pub fn is_current_logger(&self) -> bool {
        self.current == self.logger
    }

    pub fn add_buffer(&mut self, buffer: Buffer) -> BufferId {
        let buffer_id = buffer.id();
        self.buffers.insert(buffer_id, buffer);
        buffer_id
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
