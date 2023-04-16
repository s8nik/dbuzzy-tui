use anyhow::Result;
use crossterm::event::{self, KeyCode, KeyEvent};
use tui::{
    text::Text,
    widgets::{Paragraph, Widget},
};

#[derive(Debug)]
pub struct Cursor {
    pub x: u16,
    pub y: u16,
}

impl Cursor {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn set(&mut self, direction: KeyCode, lines: &Vec<String>) {
        match direction {
            KeyCode::Left => self.x = self.x.saturating_sub(1),
            KeyCode::Right => {
                if self.x.saturating_add(1) < (lines[self.y as usize].len() as u16) + 1 {
                    self.x += 1;
                }
            }
            KeyCode::Up => {
                self.y = self.y.saturating_sub(1);
                self.x = std::cmp::min(self.x, lines[self.y as usize].len() as u16);
            }
            KeyCode::Down => {
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
    pub fn handle_event(&mut self, event: KeyEvent) -> Result<bool> {
        match event {
            KeyEvent {
                code: KeyCode::Char('q'),
                modifiers: event::KeyModifiers::CONTROL,
                ..
            } => return Ok(true),
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: event::KeyModifiers::NONE | event::KeyModifiers::SHIFT,
                ..
            } => {
                let line = &mut self.lines[self.cursor.y as usize];

                line.push(c);
                self.cursor.x = line.len() as u16;
            }
            KeyEvent {
                code: direction @ (KeyCode::Up | KeyCode::Left | KeyCode::Right | KeyCode::Down),
                modifiers: event::KeyModifiers::NONE,
                ..
            } => self.cursor.set(direction, &self.lines),
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: event::KeyModifiers::NONE,
                ..
            } => {
                self.lines.push(String::new());
                self.cursor.x = 0;
                self.cursor.y = (self.lines.len() - 1) as u16;
            }
            KeyEvent {
                code: KeyCode::Backspace,
                modifiers: event::KeyModifiers::NONE,
                ..
            } => {
                let row = self.cursor.y as usize;
                if self.lines[row].is_empty() && row != 0 {
                    self.lines.pop();
                    self.cursor.y = self.cursor.y.saturating_sub(1);
                    self.cursor.x = self.lines[self.cursor.y as usize].len() as u16;
                } else {
                    let line = &mut self.lines[self.cursor.y as usize];
                    line.pop();
                    self.cursor.x = line.len() as u16;
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
