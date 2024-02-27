mod insert;
mod normal;

use std::{collections::HashMap, sync::Arc};

use crate::{
    buffer::Buffer,
    cursor::CursorMode,
    input::{Event, Input, Modifiers},
    keymap::{Bindings, Keymap},
};

pub type Callback = fn(&mut Buffer);

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

    pub fn call(&self, content: &mut Buffer) {
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
            command!(delete_char_under_cursor),
            command!(new_line),
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
    pub fn execute(&mut self, input: Input, buffer: &mut Buffer, bindings: &'static Bindings) {
        let keymap = match self.current {
            Some(node) => Some(node),
            None => bindings.get(input),
        };

        if self.current.is_none() && buffer.cursor().mode == CursorMode::Insert {
            match input {
                Input {
                    event: Event::Char('q'),
                    modifiers:
                        Modifiers {
                            ctr: true,
                            // alt: false,
                            // sup: false,
                            // hyper: false,
                            // meta: false,
                            ..
                        },
                } => self.exit = true,
                Input {
                    event: Event::Char(ch),
                    modifiers: _,
                } => insert_char(buffer, ch),
                _ => (),
            }
        }

        if let Some(node) = keymap {
            match node {
                Keymap::Leaf(command) => {
                    if let Some(command) = self.registry.get(command) {
                        command.call(buffer);
                    }
                }
                Keymap::Node(next) => self.current = next.get(input),
            }
        }
    }
}

fn insert_mode(buffer: &mut Buffer) {
    buffer.cursor_mut().mode = CursorMode::Insert;
}

fn normal_mode(buffer: &mut Buffer) {
    buffer.cursor_mut().mode = CursorMode::Normal;
}

fn insert_char(buffer: &mut Buffer, ch: char) {
    let position = buffer.cursor().position(buffer.text());

    buffer.text_mut().insert_char(position, ch);
    buffer.cursor_mut().offset += 1;
}

fn move_forward(buffer: &mut Buffer) {
    buffer.cursor_mut().offset += 1;
    cursor_line_bounds(buffer);
}

fn move_back(buffer: &mut Buffer) {
    let cursor = buffer.cursor_mut();
    cursor.offset = cursor.offset.saturating_sub(1);
}

fn move_up(buffer: &mut Buffer) {
    let cursor = buffer.cursor_mut();
    cursor.index = cursor.index.saturating_sub(1);
    cursor_line_bounds(buffer);
}

fn move_down(buffer: &mut Buffer) {
    let new_offset = buffer.cursor().index + 1;

    if new_offset < buffer.text().len_lines() {
        buffer.cursor_mut().index = new_offset;
    }

    cursor_line_bounds(buffer);
}

fn cursor_line_bounds(buffer: &mut Buffer) {
    let mut bytes_len = buffer.text().line(buffer.cursor().index).len_bytes();
    if buffer.cursor().index < buffer.text().lines().len() - 1 {
        bytes_len -= 1;
    }

    if buffer.cursor().offset > bytes_len {
        buffer.cursor_mut().offset = bytes_len;
    }
}

fn new_line(buffer: &mut Buffer) {
    insert_char(buffer, '\n');

    buffer.cursor_mut().offset = 0;
    buffer.cursor_mut().index += 1;
}

fn delete_char_under_cursor(buffer: &mut Buffer) {
    let position = buffer.cursor().position(&buffer.text());
    let bytes_len = buffer.text().line(buffer.cursor().index).len_bytes();

    if bytes_len == 0 {
        move_up(buffer);
        buffer.cursor_mut().offset = bytes_len;
    }

    if buffer.text().len_bytes() != 0 {
        buffer.text_mut().remove(position..position + 1);

        if buffer.cursor().offset == buffer.text().line(buffer.cursor().index).len_bytes() {
            move_back(buffer);
        }
    }
}

fn delete_char(buffer: &mut Buffer) {
    let position = buffer.cursor().position(&buffer.text());
    let bytes_len = buffer.text().line(buffer.cursor().index).len_bytes();

    if buffer.text().len_bytes() != 0 {
        buffer.text_mut().remove(position - 1..position);
        move_back(buffer);
    }

    if bytes_len == 0 || buffer.cursor().offset == 0 {
        move_up(buffer);
        buffer.cursor_mut().offset = buffer.text().line(buffer.cursor().index).len_bytes() - 1;
    }
}

fn move_cursor_to_line_end(buffer: &mut Buffer) {
    buffer.cursor_mut().offset = buffer.text().line(buffer.cursor().index).len_bytes();
    cursor_line_bounds(buffer);
}

fn insert_mode_line_end(buffer: &mut Buffer) {
    move_cursor_to_line_end(buffer);
    buffer.cursor_mut().mode = CursorMode::Insert;
}

fn insert_mode_line_start(buffer: &mut Buffer) {
    buffer.cursor_mut().offset = 0;
    buffer.cursor_mut().mode = CursorMode::Insert;
}

fn insert_mode_line_next(buffer: &mut Buffer) {
    move_cursor_to_line_end(buffer);
    new_line(buffer);
    buffer.cursor_mut().mode = CursorMode::Insert;
}

fn insert_mode_line_prev(buffer: &mut Buffer) {
    if buffer.cursor().index == 0 {
        buffer.cursor_mut().offset = 0;
        new_line(buffer);
        move_up(buffer);
    } else {
        move_up(buffer);
        insert_mode_line_next(buffer);
    }

    buffer.cursor_mut().mode = CursorMode::Insert;
}
