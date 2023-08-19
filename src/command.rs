use strum::EnumString;

use crate::{
    buffer::{Buffer, Content},
    cursor::CursorMode,
    input::{Event, Input},
    keymap::{Bindings, Keymap},
};

#[derive(Debug, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Command {
    InsertMode,
    NormalMode,

    MoveBack,
    MoveDown,
    MoveUp,
    MoveForward,

    InsertModeLineEnd,
    InsertModeLineStart,
    InsertModeNewLineNext,
    InsertModeNewLinePrev,

    GoToStartLine,
    GoToEndLine,

    DeleteChar,
    NewLineNext,
}

#[derive(Default)]
pub struct CommandExecutor<'a> {
    current: Option<&'a Keymap>,
}

impl<'a> CommandExecutor<'a> {
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

        if let Some(node) = keymap {
            match node {
                Keymap::Leaf(command) => match command {
                    Command::InsertMode => content.cursor.mode = CursorMode::Insert,
                    Command::NormalMode => content.cursor.mode = CursorMode::Normal,
                    Command::MoveBack => cursor_back_by(content, 1),
                    Command::MoveDown => cursor_down_by(content, 1),
                    Command::MoveUp => cursor_up_by(content, 1),
                    Command::MoveForward => cursor_forward_by(content, 1),
                    Command::InsertModeLineEnd => insert_mode_line_end(content),
                    Command::InsertModeLineStart => insert_mode_line_start(content),
                    Command::InsertModeNewLineNext => insert_mode_line_next(content),
                    Command::InsertModeNewLinePrev => insert_mode_line_prev(content),
                    Command::GoToStartLine => todo!(),
                    Command::GoToEndLine => todo!(),
                    Command::DeleteChar => delete_char(content),
                    Command::NewLineNext => new_line(content),
                },
                Keymap::Node(next) => self.current = next.get(input),
            }
        }

        if self.current.is_none() && content.cursor.mode == CursorMode::Insert {
            if let Event::Char(ch) = input.event {
                insert_char(content, ch);
            }
        }

        scroll(content, max);
    }
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

fn cursor_forward_by(content: &mut Content, offset: usize) {
    content.cursor.offset += offset;
    cursor_line_bounds(content);
}

fn cursor_back_by(content: &mut Content, offset: usize) {
    let cursor = &mut content.cursor;
    cursor.offset = cursor.offset.saturating_sub(offset);
}

fn cursor_up_by(content: &mut Content, offset: usize) {
    let cursor = &mut content.cursor;
    cursor.index = cursor.index.saturating_sub(offset);
    cursor_line_bounds(content);
}

fn cursor_down_by(content: &mut Content, offset: usize) {
    let new_offset = content.cursor.index + offset;

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
            cursor_up_by(content, 1);

            let bytes_len = content.text.line(content.cursor.index).len_bytes();
            content.cursor.offset = bytes_len;
        }

        content.text.remove(position - 1..position);
        cursor_back_by(content, 1);
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
        cursor_up_by(content, 1);
    } else {
        cursor_up_by(content, 1);
        insert_mode_line_next(content);
    }

    content.cursor.mode = CursorMode::Insert;
}
