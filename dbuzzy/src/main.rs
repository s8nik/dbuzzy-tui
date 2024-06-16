#![warn(
    clippy::perf,
    clippy::semicolon_if_nothing_returned,
    clippy::missing_const_for_fn,
    clippy::use_self
)]
use ratatui::backend::CrosstermBackend;

mod app;
mod db;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stdout = std::io::stdout();

    let backend = CrosstermBackend::new(stdout);

    let mut app = app::App::new(std::env::args(), backend)?;
    app.run().await?;

    Ok(())
}
