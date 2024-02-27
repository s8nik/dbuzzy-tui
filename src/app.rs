use std::{io::Write, time::Duration};

use anyhow::Result;
use crossterm::{execute, ExecutableCommand};
use tui::{backend::Backend, Terminal};

use crate::{cursor::CursorMode, editor::Editor};

pub struct App<B: Backend + Write> {
    editor: Editor<'static>,
    terminal: Terminal<B>,
}

impl<B: Backend + Write> App<B> {
    pub fn new(args: impl Iterator<Item = String>, backend: B) -> Result<Self> {
        let mut editor = Editor::init();

        for filepath in args.skip(1) {
            editor.open(filepath)?;
        }

        if editor.empty() {
            editor.open_scratch();
        }

        let mut terminal = Terminal::new(backend).expect("terminal");
        let size = terminal.size()?;
        editor.set_viewport(size.width, size.height);

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

    pub fn run(&mut self) -> Result<()> {
        Self::setup_panic();
        loop {
            if crossterm::event::poll(Duration::from_millis(200))? {
                let event = crossterm::event::read()?;

                if let crossterm::event::Event::Key(event) = event {
                    self.editor.handle_event(event.into());
                } else if let crossterm::event::Event::Resize(w, h) = event {
                    self.editor.set_viewport(w, h);
                }
            }

            if self.editor.exit() {
                break;
            }

            let widget = self.editor.widget();
            self.terminal.draw(|ui| {
                ui.render_widget(widget, ui.size());
            })?;

            let cursor = &self.editor.current_buff().content().cursor;

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
