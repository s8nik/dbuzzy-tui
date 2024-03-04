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
        workspace: &mut Workspace,
        bindings: &'static Bindings,
    ) -> bool {
        let Some(buffer) = current_mut!(workspace) else {
            log::error!("no active buffer");
            return false;
        };

        if buffer.cursor_mode() == CursorMode::Insert {
            match input {
                Input {
                    event: crate::input::Event::Char('q'),
                    modifiers: crate::input::Modifiers { ctr: true, .. },
                } => std::process::exit(0), // @note: for now
                Input {
                    event: crate::input::Event::Char(ch),
                    modifiers: _,
                } => {
                    insert_char(buffer, ch);
                    return true;
                }
                _ => (),
            }
        }

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
                command.call(workspace);
                executed = true;
            }

            self.current = None;
        }

        executed
    }
}

fn insert_mode(buffer: &mut Buffer) {
    buffer.update_cursor_mode(CursorMode::Insert);
}

fn normal_mode(buffer: &mut Buffer) {
    buffer.update_cursor_mode(CursorMode::Normal);
}
