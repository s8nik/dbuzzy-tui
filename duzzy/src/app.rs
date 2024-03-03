use std::io::Write;

use crossterm::{event::EventStream, execute, ExecutableCommand};
use editor::{cursor::CursorMode, editor::Editor};
use futures_util::StreamExt;
use tui::{backend::Backend, Terminal};

pub struct App<B: Backend + Write> {
    editor: Editor<'static>,
    terminal: Terminal<B>,
}

impl<B: Backend + Write> App<B> {
    pub fn new(args: impl Iterator<Item = String>, backend: B) -> anyhow::Result<Self> {
        let mut terminal = Terminal::new(backend).expect("terminal");
        let size = terminal.size()?;

        let mut editor = Editor::init(size.width as usize, size.height as usize);
        editor.open_logger();

        for filepath in args.skip(1) {
            editor.open(filepath)?;
        }

        if editor.workspace().empty() {
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

        loop {
            let render = tokio::select! {
                maybe_event = reader.next() => match maybe_event {
                    Some(Ok(event)) => self.editor.on_event(event),
                    Some(Err(e)) => {
                        log::error!("event error: {e}");
                        false
                    },
                    None => false,
                },
                Some(log) = log_rx.recv() => self.editor.on_log(log),
            };

            if self.editor.exit {
                break;
            }

            if render {
                let widget = self.editor.widget();
                self.terminal.draw(|ui| {
                    ui.render_widget(widget, ui.size());
                })?;
            }

            let cursor = &self.editor.workspace().current_buff().content().cursor;

            let x = cursor.offset as u16;
            let y = cursor.index as u16;

            self.terminal.set_cursor(x, y)?;
            execute!(self.terminal.backend_mut(), CursorMode::style(cursor.mode))?;
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
