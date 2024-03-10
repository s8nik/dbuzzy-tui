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
        (self.callback)(content)
    }
}

pub struct CommandRegistry {
    commands: HashMap<CmdType, Arc<Command>>,
}

impl CommandRegistry {
    pub fn register() -> Self {
        macro_rules! cmd {
            ($type:expr, $fun:ident $(, $($arg:expr),*)?) => {{
                Command::new($type, |workspace: &mut Workspace| {
                    $fun(workspace.current_mut(), $($($arg),*)?)
                })
            }};
            ($type:expr, $fun:ident $(, $($arg:expr),*)?, workspace) => {{
                Command::new($type, $fun)
            }};
        }

        let commands = vec![
            cmd!(CmdType::InsertMode, switch_mode, Switch::Inplace),
            cmd!(CmdType::MoveLeft, move_cursor, CursorMove::Left),
            cmd!(CmdType::MoveDown, move_cursor, CursorMove::Down(1)),
            cmd!(CmdType::MoveUp, move_cursor, CursorMove::Up(1)),
            cmd!(CmdType::MoveRight, move_cursor, CursorMove::Right),
            cmd!(CmdType::InsertModeLineEnd, switch_mode, Switch::LineEnd),
            cmd!(CmdType::InsertModeLineStart, switch_mode, Switch::LineStart),
            cmd!(CmdType::InsertModeLineNext, switch_mode, Switch::LineNext),
            cmd!(CmdType::InsertModeLinePrev, switch_mode, Switch::LinePrev),
            cmd!(CmdType::DeleteChar, delete_char),
            cmd!(CmdType::GoToTopLine, move_cursor, CursorMove::Top),
            cmd!(CmdType::GoToBottomLine, move_cursor, CursorMove::Bottom),
            cmd!(CmdType::GoToLineEnd, move_cursor, CursorMove::LineEnd),
            cmd!(CmdType::GoToLineStart, move_cursor, CursorMove::LineStart),
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
        workspace: &Workspace,
        input: Input,
    ) -> Option<Arc<Command>> {
        let buffer = workspace.current();

        let Some(bindings) = keymaps.get(&buffer.cursor_mode()) else {
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
