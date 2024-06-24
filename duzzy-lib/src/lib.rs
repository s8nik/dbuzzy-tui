pub mod colors;
pub mod event;
mod utils;

use ratatui::{buffer::Buffer, layout::Rect};
pub use utils::{ensure_config_dir, read_toml};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Component {
    Editor,
    Connections,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventOutcome {
    Render,
    Ignore,
    Exit,
    // Focus(Component),
    // Forward,
}

pub trait OnInput {
    fn on_input(&mut self, input: event::Input) -> EventOutcome;
}

pub trait Renderer {
    fn render(self, area: Rect, buf: &mut Buffer);
}
