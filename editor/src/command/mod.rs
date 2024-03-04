mod movement;
mod transform;

use std::{collections::HashMap, sync::Arc};

use movement::*;
use transform::*;

use crate::{
    buffer::{Buffer, CursorMode},
    keymap::{Bindings, Keymap},
    workspace::Workspace,
};

use super::input::Input;

pub type Callback = fn(&mut Workspace);

macro_rules! command {
    ($fun: ident) => {{
        let name = stringify!($fun);
        let command = Command::new(
            name.to_string(),
            |workspace: &mut Workspace| match workspace.current_buff_mut() {
                Some(buffer) => $fun(buffer),
                None => log::warn!("buffer is None, skipping command execution."),
            },
        );
        command
    }};
    ($fun: ident, with_workspace) => {{
        let name = stringify!($fun);
        let command = Command::new(name.to_string(), $fun);
        command
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

pub struct Registry {
    commands: HashMap<Arc<str>, Arc<Command>>,
}

impl Registry {
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

impl Default for Registry {
    fn default() -> Self {
        Self::register()
    }
}

#[derive(Default)]
pub struct CommandExecutor<'a> {
    registry: Registry,
    current: Option<&'a Keymap>,
}

impl<'a> CommandExecutor<'a> {
    pub fn execute(
        &mut self,
        input: Input,
        content: &mut Workspace,
        bindings: &'static Bindings,
    ) -> bool {
        let mut executed = false;

        self.current = match self.current {
            Some(node) => match node {
                Keymap::Leaf(_) => self.current,
                Keymap::Node(next) => next.get(input),
            },
            None => bindings.get(input),
        };

        if let Some(Keymap::Leaf(command)) = self.current {
            if let Some(command) = self.registry.get(command) {
                command.call(content);
                executed = true;
            }

            self.current = None;
        }

        executed
    }

    pub fn enter(buffer: &mut Buffer, ch: char) {
        insert_char(buffer, ch);
    }
}

fn insert_mode(buffer: &mut Buffer) {
    buffer.update_cursor_mode(CursorMode::Insert);
}

fn normal_mode(buffer: &mut Buffer) {
    buffer.update_cursor_mode(CursorMode::Normal);
}
