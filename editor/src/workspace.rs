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

    pub fn set_current(&mut self, id: BufferId) {
        self.current = id
    }

    pub fn set_logger(&mut self, id: BufferId) {
        self.logger = id
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
                "current" => $workspace.set_current(id),
                "logger" => $workspace.set_logger(id),
                _ => (),
            }
        )?
    };
}
