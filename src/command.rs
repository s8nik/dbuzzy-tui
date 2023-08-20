use std::{collections::HashMap, sync::Arc};

use crate::{
    buffer::{Buffer, Content},
    cursor::CursorMode,
    input::{Event, Input, Modifiers},
    keymap::{Bindings, Keymap},
};

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
    pub fn execute(
        &mut self,
        input: Input,
        buffer: &mut Buffer,
        bindings: &'static Bindings,
        max: usize,
    ) {
        let keymap = match self.current {
            Some(node) => Some(node),
            None => bindings.get(input),
        };

        let content = buffer.content_mut();

        if self.current.is_none() && content.cursor.mode == CursorMode::Insert {
            match input {
                Input {
                    event: Event::Char('q'),
                    modifiers:
                        Modifiers {
                            // shift: false,
                            control: true,
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
                } => insert_char(content, ch),
                _ => (),
            }
        }

        if let Some(node) = keymap {
            match node {
                Keymap::Leaf(command) => {
                    if let Some(command) = self.registry.get(command) {
                        command.call(content);
                    }
                }
                Keymap::Node(next) => self.current = next.get(input),
            }
        }

        scroll(content, max);
    }
}

pub fn insert_mode(content: &mut Content) {
    content.cursor.mode = CursorMode::Insert;
}

pub fn normal_mode(content: &mut Content) {
    content.cursor.mode = CursorMode::Normal;
}

pub fn scroll(content: &mut Content, max: usize) {
    let lower_bound = content.cursor.vscroll;
    let upper_bound = lower_bound + max - 1;

    let index = content.cursor.index;
    if index >= upper_bound {
        content.cursor.vscroll = lower_bound + index - upper_bound;
    } else if index < lower_bound {
        content.cursor.vscroll = index;
    }
}

fn insert_char(content: &mut Content, ch: char) {
    let Content { text, cursor } = content;
    let position = cursor.position(text);

    text.insert_char(position, ch);
    cursor.offset += 1;
}

fn move_forward(content: &mut Content) {
    content.cursor.offset += 1;
    cursor_line_bounds(content);
}

fn move_back(content: &mut Content) {
    let cursor = &mut content.cursor;
    cursor.offset = cursor.offset.saturating_sub(1);
}

fn move_up(content: &mut Content) {
    let cursor = &mut content.cursor;
    cursor.index = cursor.index.saturating_sub(1);
    cursor_line_bounds(content);
}

fn move_down(content: &mut Content) {
    let new_offset = content.cursor.index + 1;

    if new_offset < content.text.len_lines() {
        content.cursor.index = new_offset;
    }

    cursor_line_bounds(content);
}

fn cursor_line_bounds(content: &mut Content) {
    let Content { text, cursor } = content;

    let mut bytes_len = text.line(cursor.index).len_bytes();
    if cursor.index < text.lines().len() - 1 {
        bytes_len -= 1;
    }

    if cursor.offset > bytes_len {
        cursor.offset = bytes_len;
    }
}

fn new_line(content: &mut Content) {
    insert_char(content, '\n');

    content.cursor.offset = 0;
    content.cursor.index += 1;
}

fn delete_char(content: &mut Content) {
    let position = content.cursor.position(&content.text);

    if position != 0 {
        if content.cursor.offset == 0 {
            move_up(content);

            let bytes_len = content.text.line(content.cursor.index).len_bytes();
            content.cursor.offset = bytes_len;
        }

        content.text.remove(position - 1..position);
        move_back(content);
    }
}

fn move_cursor_to_line_end(content: &mut Content) {
    let Content { text, cursor } = content;
    cursor.offset = text.line(cursor.index).len_bytes();
    cursor_line_bounds(content);
}

fn insert_mode_line_end(content: &mut Content) {
    move_cursor_to_line_end(content);
    content.cursor.mode = CursorMode::Insert;
}

fn insert_mode_line_start(content: &mut Content) {
    content.cursor.offset = 0;
    content.cursor.mode = CursorMode::Insert;
}

fn insert_mode_line_next(content: &mut Content) {
    move_cursor_to_line_end(content);
    new_line(content);
    content.cursor.mode = CursorMode::Insert;
}

fn insert_mode_line_prev(content: &mut Content) {
    if content.cursor.index == 0 {
        content.cursor.offset = 0;
        new_line(content);
        move_up(content);
    } else {
        move_up(content);
        insert_mode_line_next(content);
    }

    content.cursor.mode = CursorMode::Insert;
}
