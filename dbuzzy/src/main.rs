use std::io::Write;

use crossterm::ExecutableCommand;
use dbuzzy::{App, Config};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stdout = std::io::stdout();

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    setup_terminal(&mut terminal)?;
    setup_panic();

    let config = Box::leak(Box::new(Config::from_toml()?));
    let mut app = App::new(config);

    app.run(&mut terminal).await?;

    clear_terminal(terminal)
}

fn setup_panic() {
    let hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |info| {
        let mut stdout = std::io::stdout();
        stdout
            .execute(crossterm::terminal::LeaveAlternateScreen)
            .ok();

        crossterm::terminal::disable_raw_mode().ok();
        hook(info);
    }));
}

fn setup_terminal<B: Backend + Write>(term: &mut Terminal<B>) -> anyhow::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        &mut term.backend_mut(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::event::EnableMouseCapture
    )?;

    Ok(())
}

fn clear_terminal<B: Backend + Write>(mut term: Terminal<B>) -> anyhow::Result<()> {
    term.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        term.backend_mut(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::event::DisableMouseCapture
    )?;

    Ok(())
}
