use std::{io::Write, time::Duration};

use crossterm::{event::EventStream, ExecutableCommand};
use duzzy_editor::{editor::Editor, renderer::EventOutcome};
use futures_util::StreamExt;
use ratatui::{backend::Backend, Terminal};

pub struct App<B: Backend + Write> {
    editor: Editor,
    terminal: Terminal<B>,
}

impl<B: Backend + Write> App<B> {
    pub fn new(args: impl Iterator<Item = String>, backend: B) -> anyhow::Result<Self> {
        Self::setup_panic();
        let mut terminal = Terminal::new(backend).expect("terminal");
        let mut editor = Editor::new();

        let mut opened = 0;
        let mut failed = 0;

        for arg in args.skip(1) {
            if let Err(e) = editor.open_file(&*arg) {
                // @todo: better logs
                // log::error!("{e}");
                failed += 1;
            } else {
                opened += 1;
            }
        }

        // @note: fix it later
        // if failed > 0 {
        // @todo: better logs
        // log::info!("Failed to open {failed} documents");
        // }

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
        let mut reader = EventStream::new();

        self.terminal.draw(|ui| {
            ui.render_widget(self.editor.widget(), ui.size());
        })?;

        loop {
            let Some(Ok(event)) = reader.next().await else {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            };

            match self.editor.on_event(event) {
                EventOutcome::Exit => break,
                EventOutcome::Render => {
                    let widget = self.editor.widget();
                    self.terminal.draw(|ui| {
                        ui.render_widget(widget, ui.size());
                    })?;
                }
                _ => (),
            };
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
