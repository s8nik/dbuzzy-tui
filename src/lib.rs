mod app;
mod editor;

use app::App;
use editor::Editor;
use tui::backend::CrosstermBackend;

pub fn run_app() -> anyhow::Result<()> {
    let stdout = std::io::stdout();

    let backend = CrosstermBackend::new(stdout);
    let editor = Editor::new();

    let mut app = App::new(editor, backend);
    app.run()?;

    Ok(())
}
