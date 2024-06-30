#![warn(
    clippy::perf,
    clippy::semicolon_if_nothing_returned,
    clippy::missing_const_for_fn,
    clippy::use_self
)]

mod app;
mod config;
pub mod db;
mod widgets;

pub use app::App;
pub use config::Config;
