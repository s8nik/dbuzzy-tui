use std::io::Write;

use crossterm::{event::EventStream, execute, ExecutableCommand};
use editor::{editor::Editor, renderer::EventOutcome};
use futures_util::StreamExt;
use tui::{backend::Backend, Terminal};

pub struct App<B: Backend + Write> {
    editor: Editor,
    terminal: Terminal<B>,
}

impl<B: Backend + Write> App<B> {
    pub fn new(args: impl Iterator<Item = String>, backend: B) -> anyhow::Result<Self> {
        let mut terminal = Terminal::new(backend).expect("terminal");
        let size = terminal.size()?;

        let mut editor = Editor::init(size.width as usize, size.height as usize);

        let mut opened = 0;
        let mut failed = 0;

        for arg in args.skip(1) {
            if let Err(e) = editor.open_file(&*arg) {
                log::error!("{e}");
                failed += 1;
            } else {
                opened += 1;
            }
        }

        if failed > 0 {
            log::info!("Failed to open {failed} documents");
        }

        if opened == 0 {
            editor.open_scratch();
        }

        crossterm::terminal::enable_raw_mode().expect("enable raw mode");
        crossterm::execute!(
            &mut terminal.backend_mut(),
            crossterm::terminal::EnterAlternateScreen,
            crossterm::event::EnableMouseCapture
        )
        .expect("enable rules");

        Ok(Self { editor, terminal })
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

    pub async fn run(&mut self) -> anyhow::Result<()> {
        Self::setup_panic();

        let (log_tx, mut log_rx) = tokio::sync::mpsc::unbounded_channel();
        editor::logger::enable(log_tx);

        let mut reader = EventStream::new();

        // first render
        let widget = self.editor.widget();
        self.terminal.draw(|ui| {
            ui.render_widget(widget, ui.size());
        })?;

        loop {
            let outcome = tokio::select! {
                Some(event) = reader.next() => match event {
                    Ok(event) => self.editor.on_event(event),
                    Err(e) => {
                        log::error!("event error: {e}");
                        continue;
                    },
                },
                Some(log) = log_rx.recv() => self.editor.on_log(log),
            };

            match outcome {
                EventOutcome::Exit => break,
                EventOutcome::Render(is_needed) if is_needed => {
                    let widget = self.editor.widget();
                    self.terminal.draw(|ui| {
                        ui.render_widget(widget, ui.size());
                    })?;
                }
                _ => (),
            };

            let cursor = self.editor.cursor();
            self.terminal.set_cursor(cursor.x, cursor.y)?;
            execute!(self.terminal.backend_mut(), cursor.style())?;
            self.terminal.show_cursor()?;
        }

        Ok(())
    }
}

impl<B: Backend + Write> Drop for App<B> {
    fn drop(&mut self) {
        self.terminal.show_cursor().expect("show cursor");
        crossterm::terminal::disable_raw_mode().expect("disable raw mode");
        crossterm::execute!(
            self.terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        )
        .expect("disable rules");
    }
}
