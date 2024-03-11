use std::collections::HashMap;

use crate::buffer::{Buffer, BufferId};

#[derive(Default)]
pub struct Workspace {
    pub(crate) buffers: HashMap<BufferId, Buffer>,
    current: BufferId,
    logger: BufferId,
}

impl Workspace {
    pub fn current(&self) -> &Buffer {
        self.buffers.get(&self.current).expect("current buff")
    }

    pub fn current_mut(&mut self) -> &mut Buffer {
        self.buffers
            .get_mut(&self.current)
            .expect("current mut buff")
    }

    pub fn logger(&mut self) -> Option<&mut Buffer> {
        self.buffers.get_mut(&self.logger)
    }

    pub const fn current_id(&self) -> BufferId {
        self.current
    }

    pub fn set_current_id(&mut self, id: BufferId) {
        self.current = id;
    }

    pub const fn logger_id(&self) -> BufferId {
        self.logger
    }

    pub fn set_logger_id(&mut self, id: BufferId) {
        self.logger = id;
    }

    pub fn logger_active(&self) -> bool {
        self.current == self.logger
    }
}

#[macro_export]
macro_rules! add_buffer {
    ($workspace:expr, $buffer:expr $(, $flag:ident)* ) => {
        let id = $buffer.id();
        $workspace.buffers.insert(id, $buffer);

        $(
            match stringify!($flag) {
                "current" => $workspace.set_current_id(id),
                "logger" => $workspace.set_logger_id(id),
                _ => (),
            }
        )?
    };
}
