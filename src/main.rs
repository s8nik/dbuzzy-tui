use tui::backend::CrosstermBackend;

use tui_editor::app::App;

fn main() -> anyhow::Result<()> {
    let stdout = std::io::stdout();

    let backend = CrosstermBackend::new(stdout);

    let mut app = App::new(std::env::args(), backend)?;
    app.run()?;

    Ok(())
}
