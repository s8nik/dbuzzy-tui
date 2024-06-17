pub mod colors;
mod utils;

use ratatui::{buffer::Buffer, layout::Rect};
pub use utils::config_toml;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventOutcome {
    Render,
    Ignore,
    Exit,
}

pub trait OnEvent {
    fn on_event(&mut self, event: crossterm::event::Event) -> EventOutcome;
}

pub trait Drawable {
    fn draw(&self, area: Rect, buf: &mut Buffer);
}

pub trait DrawableStateful {
    fn draw(&mut self, area: Rect, buf: &mut Buffer);
}
