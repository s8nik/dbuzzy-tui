#![warn(
    clippy::perf,
    clippy::semicolon_if_nothing_returned,
    clippy::missing_const_for_fn,
    clippy::use_self
)]
mod workspace;

pub mod buffer;
pub mod command;
pub mod editor;
pub mod input;
pub mod keymap;
pub mod logger;
pub mod renderer;
