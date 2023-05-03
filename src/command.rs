use crate::{
    buffer::Buffer,
    event::{Event, Input},
    keymap::{Keymap, KeymapNode},
    mode::CursorMode,
};

#[derive(Default)]
pub struct Command<'a> {
    current_node: Option<&'a KeymapNode>,
    should_exit: bool,
}

impl<'a> Command<'a> {
    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn insert_mode(&mut self, event: Input, buffer: &mut Buffer) {
        match event {
            Input {
                event: Event::Char('q'),
                ctrl: true,
                alt: false,
            } => self.should_exit = true,
            Input {
                event: Event::Esc,
                ctrl: false,
                alt: false,
            } => buffer.set_cursor_mode(CursorMode::Normal),
            Input {
                event: Event::Char(ch),
                ctrl: false,
                alt: false,
            } => Self::insert_char(buffer, ch),
            Input {
                event: direction @ (Event::Up | Event::Left | Event::Right | Event::Down),
                ctrl: false,
                alt: false,
            } => Self::move_cursor(buffer, direction),
            Input {
                event: Event::Enter,
                ctrl: false,
                alt: false,
            } => Self::new_line(buffer),
            Input {
                event: Event::Backspace,
                ctrl: false,
                alt: false,
            } => Self::delete_char(buffer),
            _ => todo!(),
        };
    }

    pub fn execute(&mut self, event: Event, buffer: &mut Buffer, keymap: &'static Keymap) {
        let keymap_node = match self.current_node {
            Some(node) => Some(node),
            None => keymap.get(event),
        };

        if let Some(node) = keymap_node {
            match node {
                KeymapNode::Leaf(command) => match command.as_str() {
                    "insert_mode" => buffer.set_cursor_mode(CursorMode::Insert),
                    "move_cursor_back" => Self::move_cursor_back_by(buffer, 1),
                    "move_cursor_down" => Self::move_cursor_down_by(buffer, 1),
                    "move_cursor_up" => Self::move_cursor_up_by(buffer, 1),
                    "move_cursor_forward" => Self::move_cursor_forward_by(buffer, 1),
                    _ => (),
                },
                KeymapNode::Node(next) => self.current_node = next.get(event),
            }
        }
    }

    fn insert_char(buffer: &mut Buffer, ch: char) {
        let pos = buffer.cursor_position();
        let text = buffer.text_mut();

        text.insert_char(pos, ch);

        let new_offset = buffer.cursor_offset() + 1;
        buffer.set_cursor_offset(new_offset);
    }

    fn move_cursor(buffer: &mut Buffer, direction: Event) {
        match direction {
            Event::Left => Self::move_cursor_back_by(buffer, 1),
            Event::Right => Self::move_cursor_forward_by(buffer, 1),
            Event::Up => Self::move_cursor_up_by(buffer, 1),
            Event::Down => Self::move_cursor_down_by(buffer, 1),
            _ => unreachable!(),
        }
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
