#![warn(
    clippy::perf,
    clippy::semicolon_if_nothing_returned,
    clippy::missing_const_for_fn,
    clippy::use_self
)]
mod buffer;
mod clipboard;
mod command;
mod document;
mod editor;
mod history;
mod keymap;
mod search;
mod selection;
mod transaction;
mod widget;

pub(crate) type SmartString = smartstring::SmartString<smartstring::LazyCompact>;

pub use editor::Editor;
pub use widget::{Cursor, EditorWidget, Viewport};
