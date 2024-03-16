mod edit;
pub mod insert;
mod shift;
mod switch;

use std::{collections::HashMap, sync::Arc};

use edit::*;
use shift::*;
use switch::*;

use crate::{
    buffer::Buffer,
    editor::Workspace,
    input::Input,
    keymap::{Keymap, Keymaps},
};

pub type Callback = fn(&mut Workspace);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd)]
pub enum CmdType {
    InsertMode,
    MoveLeft,
    MoveDown,
    MoveUp,
    MoveRight,
    InsertModeLineEnd,
    InsertModeLineStart,
    InsertModeLineNext,
    InsertModeLinePrev,
    DeleteChar,
    GoToTopLine,
    GoToBottomLine,
    GoToLineStart,
    GoToLineEnd,
}

pub struct Command {
    type_: CmdType,
    callback: Callback,
}

impl Command {
    pub fn new(type_: CmdType, callback: Callback) -> Self {
        Self { type_, callback }
    }

    pub fn call(&self, content: &mut Workspace) {
        (self.callback)(content);
    }
}

pub struct CommandRegistry {
    commands: HashMap<CmdType, Arc<Command>>,
}

impl CommandRegistry {
    pub fn register() -> Self {
        macro_rules! cmd {
            ($type:expr, $fun:ident) => {{
                Command::new($type, $fun)
            }};
        }

        let commands = vec![
            cmd!(CmdType::InsertMode, insert_mode_inplace),
            cmd!(CmdType::MoveLeft, move_left),
            cmd!(CmdType::MoveDown, move_down),
            cmd!(CmdType::MoveUp, move_up),
            cmd!(CmdType::MoveRight, move_right),
            cmd!(CmdType::InsertModeLineEnd, insert_mode_line_end),
            cmd!(CmdType::InsertModeLineStart, insert_mode_line_start),
            cmd!(CmdType::InsertModeLineNext, insert_mode_line_next),
            cmd!(CmdType::InsertModeLinePrev, insert_mode_line_prev),
            cmd!(CmdType::DeleteChar, delete_char_inplace),
            cmd!(CmdType::GoToTopLine, go_to_top_line),
            cmd!(CmdType::GoToBottomLine, go_to_bottom_line),
            cmd!(CmdType::GoToLineEnd, go_to_line_end),
            cmd!(CmdType::GoToLineStart, go_to_line_start),
        ];

        let mut map = HashMap::new();
        for command in commands {
            map.insert(command.type_, Arc::new(command));
        }

        Self { commands: map }
    }

    pub fn get(&self, type_: &CmdType) -> Option<Arc<Command>> {
        self.commands.get(type_).cloned()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::register()
    }
}

#[derive(Default)]
pub struct CommandFinder {
    registry: CommandRegistry,
    current: Option<&'static Keymap>,
}

impl CommandFinder {
    pub fn reset(&mut self) {
        self.current = None;
    }

    pub fn find(
        &mut self,
        keymaps: &'static Keymaps,
        buffer: &Buffer,
        input: Input,
    ) -> Option<Arc<Command>> {
        let Some(bindings) = keymaps.get(&buffer.mode) else {
            return None;
        };

        self.current = match self.current {
            Some(node) => match node {
                Keymap::Leaf(_) => self.current,
                Keymap::Node(next) => next.get(input),
            },
            None => bindings.get(input),
        };

        if let Some(Keymap::Leaf(command)) = self.current {
            return self.registry.get(command);
        }

        None
    }
}
