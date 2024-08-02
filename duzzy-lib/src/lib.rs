pub mod colors;
pub mod event;
mod utils;

use event::Input;
use ratatui::{buffer::Buffer, layout::Rect, widgets::ListState};
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

pub trait DuzzyListState {
    fn state(&mut self) -> &mut ListState;
    fn length(&self) -> usize;
}

pub trait DuzzyList: DuzzyListState {
    fn next(&mut self);
    fn prev(&mut self);
}

#[cfg(feature = "derive")]
pub extern crate duzzy_lib_derive;
