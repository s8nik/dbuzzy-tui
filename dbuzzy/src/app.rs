use std::{collections::HashMap, time::Duration};

use crossterm::event::{Event, EventStream};
use duzzy_editor::Editor;
use duzzy_lib::{DuzzyWidget, EventOutcome};
use futures_util::StreamExt;
use ratatui::{backend::Backend, buffer::Buffer, layout::Rect, widgets::Widget, Terminal};

use crate::{
    config::Config,
    widgets::{AppEventOutcome, AppWidget, Connections},
};

pub struct App {
    focus: AppWidget,
    editor: Box<Editor>,
    widgets: HashMap<AppWidget, Box<dyn DuzzyWidget<Outcome = AppEventOutcome>>>,
}

impl App {
    pub fn new(config: &'static Config) -> Self {
        let mut widgets: HashMap<AppWidget, Box<dyn DuzzyWidget<Outcome = AppEventOutcome>>> =
            HashMap::new();

        widgets.insert(
            AppWidget::Connections,
            Box::new(Connections::new(config.conn.as_slice())),
        );

        Self {
            widgets,
            editor: Box::new(Editor::new_scratch()),
            focus: AppWidget::Connections,
        }
    }

    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        let mut reader = EventStream::new();

        self.draw(terminal)?;

        loop {
            let Some(Ok(event)) = reader.next().await else {
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            };

            match self.handle_event(event) {
                AppEventOutcome::Outcome(event) => match event {
                    EventOutcome::Render => self.draw(terminal)?,
                    EventOutcome::Ignore => continue,
                    EventOutcome::Exit => return Ok(()),
                },
                AppEventOutcome::Focus(focus) => self.focus = focus,
            }
        }
    }

    fn draw<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> AppEventOutcome {
        let input = event.into();

        if self.focus == AppWidget::Editor {
            return self.editor.input(input).into();
        }

        self.widgets().input(input)
    }

    fn widgets(&mut self) -> &mut Box<dyn DuzzyWidget<Outcome = AppEventOutcome>> {
        self.widgets.get_mut(&self.focus).expect("should focus")
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // @note: draw widgets based on currently focused one
        self.widgets().render(area, buf);
    }
}
