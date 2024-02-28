mod movement;
mod transform;

use std::{collections::HashMap, sync::Arc};

use crate::{
    buffer::Content,
    cursor::CursorMode,
    input::{Event, Input, Modifiers},
    keymap::{Bindings, Keymap},
};

use movement::*;
use transform::*;

pub type Callback = fn(&mut Content);

macro_rules! command {
    ($fun: ident) => {{
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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn call(&self, content: &mut Content) {
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
pub struct Executor<'a> {
    registry: Registry,
    current: Option<&'a Keymap>,
    pub exit: bool,
}

impl<'a> Executor<'a> {
    pub fn execute(&mut self, input: Input, content: &mut Content, bindings: &'static Bindings) {
        let is_insert_mode = content.cursor.mode == CursorMode::Insert;

        if self.current.is_none() && is_insert_mode {
            match input {
                Input {
                    event: Event::Char('q'),
                    modifiers: Modifiers { ctr: true, .. },
                } => self.exit = true,
                Input {
                    event: Event::Char(ch),
                    modifiers: _,
                } => insert_char(content, ch),
                _ => (),
            }
        }

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
            }

            self.current = None;
        }
    }
}

fn insert_mode(content: &mut Content) {
    content.cursor.mode = CursorMode::Insert;
}

fn normal_mode(content: &mut Content) {
    content.cursor.mode = CursorMode::Normal;
}
