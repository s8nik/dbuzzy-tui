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
    ($fun: ident) => {{
        let name = stringify!($fun);
        let command = Command::new(name.to_string(), |workspace: &mut Workspace| {
            $fun(workspace.current_mut())
        });
        command
    }};
    ($fun: ident, with_workspace) => {{
        let name = stringify!($fun);
        Command::new(name.to_string(), $fun)
    }};
}
pub struct Command {
    name: String,
    callback: Callback,
}

impl Command {
    pub fn new(name: String, callback: Callback) -> Self {
        Self { name, callback }
    }

    pub fn call(&self, content: &mut Workspace) {
        (self.callback)(content)
    }
}

pub struct CommandRegistry {
    commands: HashMap<Arc<str>, Arc<Command>>,
}

impl CommandRegistry {
    pub fn register() -> Self {
        let commands = vec![
            command!(insert_mode),
            command!(normal_mode),
            command!(move_back),
            command!(move_down),
            command!(move_up),
            command!(move_forward),
            command!(insert_mode_line_end),
            command!(insert_mode_line_start),
            command!(insert_mode_line_next),
            command!(insert_mode_line_prev),
            command!(delete_char),
            command!(delete_char_backspace),
            command!(new_line),
            command!(go_to_start_line),
            command!(go_to_end_line),
            command!(go_to_start_curr_line),
            command!(go_to_end_curr_line),
        ];

        let mut map = HashMap::new();
        for command in commands {
            map.insert(Arc::from(command.name.as_str()), Arc::new(command));
        }

        Self { commands: map }
    }

    pub fn get(&self, name: &str) -> Option<Arc<Command>> {
        self.commands.get(name).cloned()
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
