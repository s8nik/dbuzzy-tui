mod adjustment;
mod history;
pub mod insert_mode;
mod movement;
mod switch_mode;

use std::{collections::HashMap, sync::Arc};

use adjustment::*;
use history::{redo, undo};
use movement::*;
use switch_mode::*;

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
    Undo,
    Redo,
    NormalMode,
    VisualMode,
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
        let commands = vec![
            Command::new(CmdType::InsertMode, insert_mode_inplace),
            Command::new(CmdType::MoveLeft, move_left),
            Command::new(CmdType::MoveDown, move_down),
            Command::new(CmdType::MoveUp, move_up),
            Command::new(CmdType::MoveRight, move_right),
            Command::new(CmdType::InsertModeLineEnd, insert_mode_line_end),
            Command::new(CmdType::InsertModeLineStart, insert_mode_line_start),
            Command::new(CmdType::InsertModeLineNext, insert_mode_line_next),
            Command::new(CmdType::InsertModeLinePrev, insert_mode_line_prev),
            Command::new(CmdType::DeleteChar, delete_char_inplace),
            Command::new(CmdType::GoToTopLine, go_to_top_line),
            Command::new(CmdType::GoToBottomLine, go_to_bottom_line),
            Command::new(CmdType::GoToLineEnd, go_to_line_end),
            Command::new(CmdType::GoToLineStart, go_to_line_start),
            Command::new(CmdType::Undo, undo),
            Command::new(CmdType::Redo, redo),
            Command::new(CmdType::VisualMode, visual_mode),
            Command::new(CmdType::NormalMode, normal_mode),
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
        let bindings = keymaps.get(&buffer.mode())?;

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
