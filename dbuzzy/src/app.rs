use std::{collections::HashMap, time::Duration};

use crossterm::event::{Event, EventStream};
use duzzy_editor::Editor;
use duzzy_lib::{DuzzyWidget, EventOutcome, NamedWidget};
use futures_util::StreamExt;
use ratatui::{backend::Backend, buffer::Buffer, layout::Rect, widgets::Widget, Terminal};

use crate::{config::Config, widgets::Connections};

pub struct App {
    components: HashMap<&'static str, Box<dyn DuzzyWidget>>,
    focus: &'static str,
}

impl App {
    pub fn new(config: &'static Config) -> Self {
        let mut components: HashMap<&str, Box<dyn DuzzyWidget>> = HashMap::new();

        components.insert(
            Connections::name(),
            Box::new(Connections::new(config.conn.as_slice())),
        );

        components.insert(Editor::name(), Box::new(Editor::new_scratch()));

        Self {
            components,
            focus: Connections::name(),
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
                EventOutcome::Render => self.draw(terminal)?,
                EventOutcome::Exit => return Ok(()),
                EventOutcome::Ignore => continue,
                EventOutcome::Focus(_) => todo!(),
            }
        }
    }

    fn draw<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> EventOutcome {
        self.focused().input(event.into())
    }

    fn focused(&mut self) -> &mut Box<dyn DuzzyWidget> {
        self.components.get_mut(self.focus).expect("should focus")
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.focused().render(area, buf);
    }
}
