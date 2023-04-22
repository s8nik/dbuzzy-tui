use std::{collections::HashMap, path::Path};

use anyhow::Result;
use tui::{
    text::Text,
    widgets::{Paragraph, Widget},
};

use crate::{
    buffer::{Buffer, BufferId},
    event::{Event, Input},
    mode::CursorMode,
};

#[derive(Default)]
pub struct Editor {
    buffers: HashMap<BufferId, Buffer>,
    current: BufferId,
    exit: bool,
}

impl Editor {
    pub fn init() -> Self {
        Self {
            buffers: HashMap::new(),
            current: BufferId::MAX,
            exit: false,
        }
    }

    pub fn exit(&self) -> bool {
        self.exit
    }

    pub fn current_buff(&self) -> &Buffer {
        self.buffers.get(&self.current).expect("should exist")
    }

    pub fn current_buff_mut(&mut self) -> &mut Buffer {
        self.buffers.get_mut(&self.current).expect("should exist")
    }

    pub fn empty(&self) -> bool {
        self.current == BufferId::MAX
    }

    pub fn open(&mut self, filepath: impl AsRef<Path>) -> Result<()> {
        let buffer = Buffer::from_path(filepath)?;

        self.add_buffer(buffer);

        Ok(())
    }

    pub fn open_scratch(&mut self) {
        let buffer = Buffer::default();

        self.add_buffer(buffer);
    }

    fn add_buffer(&mut self, buffer: Buffer) {
        let buffer_id = buffer.id();
        self.buffers.insert(buffer_id, buffer);
        self.current = buffer_id;
    }

    pub fn handle_event(&mut self, event: Input) -> Result<()> {
        let cursor_mode = self.current_buff().cursor_mode();
        match cursor_mode {
            CursorMode::Insert => self.handle_insert_mode_event(event)?,
            CursorMode::Normal => self.handle_normal_mode_event(event)?,
            CursorMode::Visual => todo!(),
        }

        Ok(())
    }

    fn handle_normal_mode_event(&mut self, event: Input) -> Result<()> {
        match event {
            Input {
                event: Event::Char('i'),
                ctrl: false,
                alt: false,
            } => self.current_buff_mut().set_cursor_mode(CursorMode::Insert),
            Input {
                event: Event::Char(direction @ ('h' | 'j' | 'k' | 'l')),
                ctrl: false,
                alt: false,
            } => match direction {
                'h' => self.current_buff_mut().move_back_by(1),
                'j' => self.current_buff_mut().move_down_by(1),
                'k' => self.current_buff_mut().move_up_by(1),
                'l' => self.current_buff_mut().move_forward_by(1),
                _ => unreachable!(),
            },
            _ => todo!(),
        }

        Ok(())
    }

    fn handle_insert_mode_event(&mut self, event: Input) -> Result<()> {
        match event {
            Input {
                event: Event::Char('q'),
                ctrl: true,
                alt: false,
            } => self.exit = true,
            Input {
                event: Event::Esc,
                ctrl: false,
                alt: false,
            } => self.current_buff_mut().set_cursor_mode(CursorMode::Normal),
            Input {
                event: Event::Char(c),
                ctrl: false,
                alt: false,
            } => {
                self.current_buff_mut().insert_char(c);
                self.current_buff_mut().move_forward_by(1);
            }
            Input {
                event: direction @ (Event::Up | Event::Left | Event::Right | Event::Down),
                ctrl: false,
                alt: false,
            } => match direction {
                Event::Left => self.current_buff_mut().move_back_by(1),
                Event::Right => self.current_buff_mut().move_forward_by(1),
                Event::Up => self.current_buff_mut().move_up_by(1),
                Event::Down => self.current_buff_mut().move_down_by(1),
                _ => unreachable!(),
            },
            Input {
                event: Event::Enter,
                ctrl: false,
                alt: false,
            } => self.current_buff_mut().new_line(),
            Input {
                event: Event::Backspace,
                ctrl: false,
                alt: false,
            } => self.current_buff_mut().backspace(),
            _ => todo!(),
        };

        Ok(())
    }

    pub fn widget(&self) -> Renderer {
        Renderer(self)
    }

    pub fn text(&self) -> Text {
        Text::raw(self.current_buff().text())
    }

    pub fn cursor(&self) -> (usize, usize) {
        let buffer = self.current_buff();
        (buffer.cursor_offset(), buffer.line_index())
    }
}

pub struct Renderer<'a>(&'a Editor);

impl<'a> Widget for Renderer<'a> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let inner = Paragraph::new(self.0.text());
        inner.render(area, buf);
    }
}
