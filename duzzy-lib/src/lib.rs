pub mod colors;
pub mod event;
mod utils;

use event::Input;
use ratatui::{buffer::Buffer, layout::Rect};
pub use utils::{ensure_config_dir, read_toml};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventOutcome {
    Render,
    Ignore,
    Exit,
}

pub trait DuzzyWidget {
    type Outcome;

    fn input(&mut self, input: Input) -> Self::Outcome;
    fn render(&mut self, area: Rect, buf: &mut Buffer);
}
