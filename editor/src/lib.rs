#![warn(
    clippy::perf,
    clippy::semicolon_if_nothing_returned,
    clippy::missing_const_for_fn,
    clippy::use_self
)]

mod buffer;
mod command;
mod document;
pub mod editor;
mod history;
mod input;
mod keymap;
pub mod logger;
pub mod renderer;
