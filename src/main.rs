use tui::backend::CrosstermBackend;

use tui_editor::{app::App, editor::Editor};

fn main() -> anyhow::Result<()> {
    let stdout = std::io::stdout();

    let backend = CrosstermBackend::new(stdout);
    let editor = Editor::default();

    let mut app = App::new(editor, backend);
    app.run()?;

    Ok(())
}
