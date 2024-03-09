pub mod insert;
mod movement;
mod switch;
mod transform;

use std::{collections::HashMap, sync::Arc};

use movement::*;
use switch::*;
use transform::*;

use crate::{
    input::Input,
    keymap::{Keymap, Keymaps},
    workspace::Workspace,
};

pub type Callback = fn(&mut Workspace);

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, PartialOrd)]
pub enum CommandType {
    InsertMode,
    NormalMode,
    MoveBack,
    MoveDown,
    MoveUp,
    MoveForward,
    InsertModeLineEnd,
    InsertModeLineStart,
    InsertModeLineNext,
    InsertModeLinePrev,
    DeleteChar,
    DeleteCharBackspace,
    NewLine,
    GoToStartLine,
    GoToEndLine,
    GoToStartCurrLine,
    GoToEndCurrLine,
}

pub struct Command {
    type_: CommandType,
    callback: Callback,
}

impl Command {
    pub fn new(type_: CommandType, callback: Callback) -> Self {
        Self { type_, callback }
    }

    pub fn call(&self, content: &mut Workspace) {
        (self.callback)(content)
    }
}

pub struct CommandRegistry {
    commands: HashMap<CommandType, Arc<Command>>,
}

impl CommandRegistry {
    pub fn register() -> Self {
        macro_rules! command {
            ($type: expr, $fun: ident) => {{
                Command::new($type, |workspace: &mut Workspace| {
                    $fun(workspace.current_mut())
                })
            }};
            ($type: expr, $fun: ident, workspace) => {{
                Command::new($type, $fun)
            }};
        }

        let commands = vec![
            command!(CommandType::InsertMode, insert_mode),
            command!(CommandType::NormalMode, normal_mode),
            command!(CommandType::MoveBack, move_back),
            command!(CommandType::MoveDown, move_down),
            command!(CommandType::MoveUp, move_up),
            command!(CommandType::MoveForward, move_forward),
            command!(CommandType::InsertModeLineEnd, insert_mode_line_end),
            command!(CommandType::InsertModeLineStart, insert_mode_line_start),
            command!(CommandType::InsertModeLineNext, insert_mode_line_next),
            command!(CommandType::InsertModeLinePrev, insert_mode_line_prev),
            command!(CommandType::DeleteChar, delete_char),
            command!(CommandType::DeleteCharBackspace, delete_char_backspace),
            command!(CommandType::NewLine, new_line),
            command!(CommandType::GoToStartLine, go_to_start_line),
            command!(CommandType::GoToEndLine, go_to_end_line),
            command!(CommandType::GoToStartCurrLine, go_to_start_curr_line),
            command!(CommandType::GoToEndCurrLine, go_to_end_curr_line),
        ];

        let mut map = HashMap::new();
        for command in commands {
            map.insert(command.type_, Arc::new(command));
        }

        Self { commands: map }
    }

    pub fn get(&self, type_: &CommandType) -> Option<Arc<Command>> {
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
        workspace: &Workspace,
        input: Input,
    ) -> Option<Arc<Command>> {
        let buffer = workspace.current();

        let bindings = keymaps
            .get(&buffer.cursor_mode())
            .expect("keymap must be registered");

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
