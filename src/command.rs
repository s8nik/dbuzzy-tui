use strum::EnumString;

use crate::{
    buffer::Buffer,
    input::Input,
    keymap::{Bindings, Keymap},
    mode::CursorMode,
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
    pub exit: bool,
}

impl<'a> CommandExecutor<'a> {
    pub fn free(&self) -> bool {
        self.current.is_none()
    }

    pub fn execute(&mut self, input: Input, buffer: &mut Buffer, bindings: &'static Bindings) {
        let keymap = match self.current {
            Some(node) => Some(node),
            None => bindings.get(input),
        };

        if let Some(node) = keymap {
            match node {
                Keymap::Leaf(command) => match command {
                    Command::InsertMode => buffer.set_cursor_mode(CursorMode::Insert),
                    Command::NormalMode => buffer.set_cursor_mode(CursorMode::Normal),
                    Command::MoveBack => cursor_back_by(buffer, 1),
                    Command::MoveDown => cursor_down_by(buffer, 1),
                    Command::MoveUp => cursor_up_by(buffer, 1),
                    Command::MoveForward => cursor_forward_by(buffer, 1),
                    Command::InsertModeLineEnd => insert_mode_line_end(buffer),
                    Command::InsertModeLineStart => insert_mode_line_start(buffer),
                    Command::InsertModeNewLineNext => insert_mode_line_next(buffer),
                    Command::InsertModeNewLinePrev => insert_mode_line_prev(buffer),
                    Command::GoToStartLine => todo!(),
                    Command::GoToEndLine => todo!(),
                    Command::DeleteChar => delete_char(buffer),
                    Command::NewLineNext => new_line(buffer),
                },
                Keymap::Node(next) => self.current = next.get(input),
            }
        }
    }
}

pub fn scroll(buffer: &mut Buffer, max: usize) {
    let lower_bound = buffer.vscroll_index();
    let upper_bound = lower_bound + max - 1;

    let line_index = buffer.line_index();
    if line_index >= upper_bound {
        buffer.set_vsscroll_index(lower_bound + line_index - upper_bound);
    } else if line_index < lower_bound {
        buffer.set_vsscroll_index(line_index)
    }
}

fn insert_char(buffer: &mut Buffer, ch: char) {
    let pos = buffer.cursor_position();
    let text = buffer.text_mut();

    text.insert_char(pos, ch);

    let new_offset = buffer.cursor_offset() + 1;
    buffer.set_cursor_offset(new_offset);
}

fn cursor_forward_by(buffer: &mut Buffer, offset: usize) {
    let new_offset = buffer.cursor_offset() + offset;
    buffer.set_cursor_offset(new_offset);
    cursor_line_bounds(buffer);
}

fn cursor_back_by(buffer: &mut Buffer, offset: usize) {
    let cursor_offset = buffer.cursor_offset();
    let new_offset = cursor_offset.saturating_sub(offset);
    buffer.set_cursor_offset(new_offset);
}

fn cursor_up_by(buffer: &mut Buffer, offset: usize) {
    let line_index = buffer.line_index();
    let new_line_index = line_index.saturating_sub(offset);
    buffer.set_line_index(new_line_index);
    cursor_line_bounds(buffer);
}

fn cursor_down_by(buffer: &mut Buffer, offset: usize) {
    let new_offset = buffer.line_index() + offset;

    if new_offset < buffer.text().len_lines() {
        buffer.set_line_index(new_offset);
    }

    cursor_line_bounds(buffer);
}

fn cursor_line_bounds(buffer: &mut Buffer) {
    let text = buffer.text();

    let mut line_bytes_len = text.line(buffer.line_index()).len_bytes();
    if buffer.line_index() < text.lines().len() - 1 {
        line_bytes_len -= 1;
    }

    if buffer.cursor_offset() > line_bytes_len {
        buffer.set_cursor_offset(line_bytes_len);
    }
}

fn new_line(buffer: &mut Buffer) {
    insert_char(buffer, '\n');

    buffer.set_cursor_offset(0);
    buffer.set_line_index(buffer.line_index() + 1);
}

fn delete_char(buffer: &mut Buffer) {
    let pos = buffer.cursor_position();

    if pos != 0 {
        if buffer.cursor_offset() == 0 {
            cursor_up_by(buffer, 1);

            let new_offset = buffer.text().line(buffer.line_index()).len_bytes();
            buffer.set_cursor_offset(new_offset);
        }

        buffer.text_mut().remove(pos - 1..pos);
        cursor_back_by(buffer, 1);
    }
}

fn move_cursor_to_line_end(buffer: &mut Buffer) {
    let new_offset = buffer.text().line(buffer.line_index()).len_bytes();
    buffer.set_cursor_offset(new_offset);
    cursor_line_bounds(buffer);
}

fn insert_mode_line_end(buffer: &mut Buffer) {
    move_cursor_to_line_end(buffer);
    buffer.set_cursor_mode(CursorMode::Insert);
}

fn insert_mode_line_start(buffer: &mut Buffer) {
    buffer.set_cursor_offset(0);
    buffer.set_cursor_mode(CursorMode::Insert);
}

fn insert_mode_line_next(buffer: &mut Buffer) {
    move_cursor_to_line_end(buffer);
    new_line(buffer);
    buffer.set_cursor_mode(CursorMode::Insert);
}

fn insert_mode_line_prev(buffer: &mut Buffer) {
    if buffer.line_index() == 0 {
        buffer.set_cursor_offset(0);
        new_line(buffer);
        cursor_up_by(buffer, 1);
    } else {
        cursor_up_by(buffer, 1);
        insert_mode_line_next(buffer);
    }

    buffer.set_cursor_mode(CursorMode::Insert);
}
