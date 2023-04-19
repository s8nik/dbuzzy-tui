use anyhow::Result;
use tui::{
    text::Text,
    widgets::{Paragraph, Widget},
};

use crate::{
    buffer::Buffer,
    event::{Event, Input},
};

#[derive(Default)]
pub struct Editor {
    pub buffer: Buffer,
}

impl Editor {
    // todo: change to vec of buffers later
    pub fn new(buffer: Buffer) -> Self {
        Self {
            buffer,
            ..Default::default()
        }
    }

    pub fn handle_event(&mut self, event: Input) -> Result<bool> {
        match event {
            Input {
                event: Event::Char('q'),
                ctrl: true,
                alt: false,
            } => return Ok(true),
            Input {
                event: Event::Char(c),
                ctrl: false,
                alt: false,
            } => {
                self.buffer.insert_char(c);
                self.buffer.move_forward_by(1);
            }
            Input {
                event: direction @ (Event::Up | Event::Left | Event::Right | Event::Down),
                ctrl: false,
                alt: false,
            } => match direction {
                Event::Left => self.buffer.move_back_by(1),
                Event::Right => self.buffer.move_forward_by(1),
                Event::Up => self.buffer.move_up_by(1),
                Event::Down => self.buffer.move_down_by(1),
                _ => unreachable!(),
            },
            Input {
                event: Event::Enter,
                ctrl: false,
                alt: false,
            } => self.buffer.new_line(),
            Input {
                event: Event::Backspace,
                ctrl: false,
                alt: false,
            } => self.buffer.backspace(),
            _ => todo!(),
        };

        Ok(false)
    }

    pub fn widget(&self) -> Renderer {
        Renderer(self)
    }

    pub fn text(&self) -> Text {
        Text::raw(self.buffer.text())
    }

    pub fn cursor(&self) -> (usize, usize) {
        (self.buffer.cursor_offset(), self.buffer.line_index())
    }
}

pub struct Renderer<'a>(&'a Editor);

impl<'a> Widget for Renderer<'a> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let inner = Paragraph::new(self.0.text());
        inner.render(area, buf);
    }
}
