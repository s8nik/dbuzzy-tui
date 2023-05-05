use strum::EnumString;

use crate::{
    buffer::Buffer,
    event::Input,
    keymap::{Keymap, KeymapNode},
    mode::CursorMode,
};

#[derive(Debug, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum CommandType {
    SwitchToInsertMode,
    SwitchToNormalMode,

    MoveBack,
    MoveDown,
    MoveUp,
    MoveForward,

    SwitchToInsertModeLineEnd,
    SwitchToInsertModeLineStart,

    GoToStartLine,
    GoToEndLine,

    DeleteChar,
    NewLine,
}

#[derive(Default)]
pub struct Command<'a> {
    current_node: Option<&'a KeymapNode>,
    should_exit: bool,
}

impl<'a> Command<'a> {
    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn do_exit(&mut self) {
        self.should_exit = true
    }

    // TODO: Handle ctrl/alt
    pub fn execute(&mut self, input: Input, buffer: &mut Buffer, keymap: &'static Keymap) {
        let keymap_node = match self.current_node {
            Some(node) => Some(node),
            None => keymap.get(input.event),
        };

        if let Some(node) = keymap_node {
            match node {
                KeymapNode::Leaf(command_type) => match command_type {
                    CommandType::SwitchToInsertMode => buffer.set_cursor_mode(CursorMode::Insert),
                    CommandType::SwitchToNormalMode => buffer.set_cursor_mode(CursorMode::Normal),
                    CommandType::MoveBack => Self::move_cursor_back_by(buffer, 1),
                    CommandType::MoveDown => Self::move_cursor_down_by(buffer, 1),
                    CommandType::MoveUp => Self::move_cursor_up_by(buffer, 1),
                    CommandType::MoveForward => Self::move_cursor_forward_by(buffer, 1),
                    CommandType::SwitchToInsertModeLineEnd => todo!(),
                    CommandType::SwitchToInsertModeLineStart => todo!(),
                    CommandType::GoToStartLine => todo!(),
                    CommandType::GoToEndLine => todo!(),
                    CommandType::DeleteChar => Self::delete_char(buffer),
                    CommandType::NewLine => Self::new_line(buffer),
                },
                KeymapNode::Node(next) => self.current_node = next.get(input.event),
            }
        }
    }

    pub fn insert_char(buffer: &mut Buffer, ch: char) {
        let pos = buffer.cursor_position();
        let text = buffer.text_mut();

        text.insert_char(pos, ch);

        let new_offset = buffer.cursor_offset() + 1;
        buffer.set_cursor_offset(new_offset);
    }

    fn move_cursor_forward_by(buffer: &mut Buffer, offset: usize) {
        let new_offset = buffer.cursor_offset() + offset;
        buffer.set_cursor_offset(new_offset);
        Self::cursor_line_bounds(buffer);
    }

    fn move_cursor_back_by(buffer: &mut Buffer, offset: usize) {
        let cursor_offset = buffer.cursor_offset();
        let new_offset = cursor_offset.saturating_sub(offset);
        buffer.set_cursor_offset(new_offset);
    }

    fn move_cursor_up_by(buffer: &mut Buffer, offset: usize) {
        let line_index = buffer.line_index();
        let new_line_index = line_index.saturating_sub(offset);
        buffer.set_line_index(new_line_index);
        Self::cursor_line_bounds(buffer);
    }

    fn move_cursor_down_by(buffer: &mut Buffer, offset: usize) {
        let new_offset = buffer.line_index() + offset;

        if new_offset < buffer.text().len_lines() {
            buffer.set_line_index(new_offset);
        }

        Self::cursor_line_bounds(buffer);
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
        Self::insert_char(buffer, '\n');

        buffer.set_cursor_offset(0);
        buffer.set_line_index(buffer.line_index() + 1);
    }

    fn delete_char(buffer: &mut Buffer) {
        let pos = buffer.cursor_position();

        if pos != 0 {
            if buffer.cursor_offset() == 0 {
                Self::move_cursor_up_by(buffer, 1);

                let new_offset = buffer.text().line(buffer.line_index()).len_bytes();
                buffer.set_cursor_offset(new_offset);
            }

            buffer.text_mut().remove(pos - 1..pos);
            Self::move_cursor_back_by(buffer, 1);
        }
    }
}
