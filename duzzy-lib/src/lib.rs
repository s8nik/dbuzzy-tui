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
    Focus(&'static str),
}

pub trait DuzzyWidget {
    fn input(&mut self, input: Input) -> EventOutcome;
    fn render(&mut self, area: Rect, buf: &mut Buffer);
}

pub trait NamedWidget: DuzzyWidget {
    fn name() -> &'static str;
}
