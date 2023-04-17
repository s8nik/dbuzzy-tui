use anyhow::Result;
use tui::{
    text::Text,
    widgets::{Paragraph, Widget},
};

use crate::event::{Event, Input};

#[derive(Debug)]
pub struct Cursor {
    pub x: u16,
    pub y: u16,
}

impl Cursor {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn set(&mut self, direction: Event, lines: &Vec<String>) {
        match direction {
            Event::Left => self.x = self.x.saturating_sub(1),
            Event::Right => {
                if self.x.saturating_add(1) < (lines[self.y as usize].len() as u16) + 1 {
                    self.x += 1;
                }
            }
            Event::Up => {
                self.y = self.y.saturating_sub(1);
                self.x = std::cmp::min(self.x, lines[self.y as usize].len() as u16);
            }
            Event::Down => {
                if self.y.saturating_add(1) < lines.len() as u16 {
                    self.y += 1;
                    self.x = std::cmp::min(self.x, lines[self.y as usize].len() as u16);
                }
            }
            _ => unimplemented!(),
        }
    }
}

pub struct Editor {
    pub cursor: Cursor,
    pub lines: Vec<String>,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            cursor: Cursor::new(0, 0),
            lines: vec![String::new()],
        }
    }
}

impl Editor {
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
                let line = &mut self.lines[self.cursor.y as usize];

                line.insert(self.cursor.x as usize, c);
                self.cursor.x = self.cursor.x.saturating_add(1);
            }
            Input {
                event: direction @ (Event::Up | Event::Left | Event::Right | Event::Down),
                ctrl: false,
                alt: false,
            } => self.cursor.set(direction, &self.lines),
            Input {
                event: Event::Enter,
                ctrl: false,
                alt: false,
            } => {
                let x = self.cursor.x as usize;
                let line = &mut self.lines[self.cursor.y as usize];

                self.cursor.x = 0;
                self.cursor.y = self.cursor.y.saturating_add(1);

                if x < line.len() {
                    let str_to_move = line.split_off(x);
                    self.lines.insert(self.cursor.y as usize, str_to_move);
                } else {
                    self.lines.insert(self.cursor.y as usize, String::new());
                }
            }
            Input {
                event: Event::Backspace,
                ctrl: false,
                alt: false,
            } => {
                if self.cursor.x == 0 {
                    let curr_y = self.cursor.y as usize;
                    if curr_y != 0 {
                        self.cursor.y = self.cursor.y.saturating_sub(1);
                        self.cursor.x = self.lines[self.cursor.y as usize].len() as u16;

                        let prev = self.lines.remove(curr_y);
                        if !prev.is_empty() {
                            self.lines[self.cursor.y as usize].push_str(&prev);
                        }
                    }
                } else {
                    let line = &mut self.lines[self.cursor.y as usize];
                    let mut chars: Vec<char> = line.chars().collect();
                    chars.remove((self.cursor.x - 1) as usize);
                    *line = chars.into_iter().collect();
                    self.cursor.x = self.cursor.x.saturating_sub(1);
                }
            }
            _ => todo!(),
        };

        Ok(false)
    }

    pub fn widget(&self) -> Renderer {
        Renderer(self)
    }

    pub fn text(&self) -> Text {
        Text::from(self.lines.join("\n"))
    }

    pub fn cursor(&self) -> (u16, u16) {
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
