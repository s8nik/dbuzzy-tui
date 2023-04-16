use std::{io::Write, time::Duration};

use tui::{backend::Backend, Terminal};

use crate::editor::Editor;

pub struct App<B: Backend + Write> {
    editor: Editor,
    terminal: Terminal<B>,
}

impl<B: Backend + Write> App<B> {
    pub fn new(editor: Editor, backend: B) -> Self {
        let mut terminal = Terminal::new(backend).expect("terminal");

        if cfg!(feature = "crossterm") {
            crossterm::terminal::enable_raw_mode().expect("enable raw mode");
            crossterm::execute!(
                &mut terminal.backend_mut(),
                crossterm::terminal::EnterAlternateScreen,
                crossterm::event::EnableMouseCapture
            )
            .expect("enable rules");
        }

        Self { editor, terminal }
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        loop {
            let exit = {
                if crossterm::event::poll(Duration::from_millis(200))? {
                    if let crossterm::event::Event::Key(event) = crossterm::event::read()? {
                        self.editor.handle_event(event)?
                    } else {
                        false
                    }
                } else {
                    false
                }
            };

            if exit {
                break;
            }

            let widget = self.editor.widget();
            self.terminal.draw(|ui| {
                ui.render_widget(widget, ui.size());
            })?;

            let (x, y) = self.editor.cursor();
            self.terminal.set_cursor(x, y)?;
            self.terminal.show_cursor()?;
        }

        Ok(())
    }
}

impl<B: Backend + Write> Drop for App<B> {
    fn drop(&mut self) {
        self.terminal.show_cursor().expect("show cursor");
        if cfg!(feature = "crossterm") {
            crossterm::terminal::disable_raw_mode().expect("disable raw mode");
            crossterm::execute!(
                self.terminal.backend_mut(),
                crossterm::terminal::LeaveAlternateScreen,
                crossterm::event::DisableMouseCapture
            )
            .expect("disable rules");
        }
    }
}
