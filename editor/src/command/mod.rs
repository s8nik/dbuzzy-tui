mod movement;
mod transform;

use std::{collections::HashMap, sync::Arc};

use movement::*;
use transform::*;

use crate::{
    buffer::{Buffer, CursorMode},
    input::Input,
    keymap::{Keymap, Keymaps},
    renderer::EventOutcome,
    workspace::Workspace,
};

pub type Callback = fn(&mut Workspace);

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

#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd)]
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
pub struct CommandResolver {
    registry: CommandRegistry,
    current: Option<&'static Keymap>,
}

impl CommandResolver {
    pub fn reset(&mut self) {
        self.current = None;
    }

    pub fn resolve(
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

pub fn insert_mode_on_key(buffer: &mut Buffer, input: Input) -> EventOutcome {
    match input {
        Input {
            event: crate::input::Event::Char('q'),
            modifiers: crate::input::Modifiers { ctr: true, .. },
        } => EventOutcome::Exit,
        Input {
            event: crate::input::Event::Char(ch),
            modifiers: _,
        } => {
            insert_char(buffer, ch);
            EventOutcome::Render(true)
        }
        _ => EventOutcome::Render(false),
    }
}

fn insert_mode(buffer: &mut Buffer) {
    buffer.update_cursor_mode(CursorMode::Insert);
}

fn normal_mode(buffer: &mut Buffer) {
    buffer.update_cursor_mode(CursorMode::Normal);
}
