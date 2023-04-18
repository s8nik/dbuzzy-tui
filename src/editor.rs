use anyhow::Result;
use ropey::Rope;
use tui::{
    text::Text,
    widgets::{Paragraph, Widget},
};

use crate::{
    buffer::Buffer,
    event::{Event, Input},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
}

impl Cursor {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn set(&mut self, direction: Event, text: &Rope) {
        match direction {
            Event::Left => self.x = self.x.saturating_sub(1),
            Event::Right => {
                if self.x.saturating_add(1) < (text.line(self.y).len_chars()) + 1 {
                    self.x += 1;
                }
            }
            Event::Up => {
                self.y = self.y.saturating_sub(1);
                self.x = std::cmp::min(self.x, text.line(self.y).len_chars());
            }
            Event::Down => {
                if self.y.saturating_add(1) < text.len_lines() {
                    self.y += 1;
                    self.x = std::cmp::min(self.x, text.line(self.y).len_chars());
                }
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Default)]
pub struct Editor {
    pub cursor: Cursor,
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
                let text = self.buffer.text_mut();
                let curr_pos = text.line_to_char(self.cursor.y) + self.cursor.x;

                text.insert_char(curr_pos, c);
                self.cursor.x = self.cursor.x.saturating_add(1);
            }
            Input {
                event: direction @ (Event::Up | Event::Left | Event::Right | Event::Down),
                ctrl: false,
                alt: false,
            } => self.cursor.set(direction, self.buffer.text()),
            Input {
                event: Event::Enter,
                ctrl: false,
                alt: false,
            } => {
                self.buffer.text_mut().split_off(self.cursor.x);

                self.cursor.x = 0;
                self.cursor.y = self.cursor.y.saturating_add(1);
            }
            Input {
                event: Event::Backspace,
                ctrl: false,
                alt: false,
            } => {
                todo!()
            }
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
        (self.cursor.x, self.cursor.y)
    }
}

pub struct Renderer<'a>(&'a Editor);

impl<'a> Widget for Renderer<'a> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        let inner = Paragraph::new(self.0.text());
        inner.render(area, buf);
    }
}
