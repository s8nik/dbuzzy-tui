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
mod event;
mod history;
mod keymap;
pub mod renderer;
mod selection;
mod transaction;

pub type SmartString = smartstring::SmartString<smartstring::LazyCompact>;
