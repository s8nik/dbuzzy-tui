use std::time::Duration;

use crossterm::event::{Event, EventStream};
use duzzy_lib::{DrawableStateful, EventOutcome, OnInput};
use futures_util::StreamExt;
use ratatui::{backend::Backend, buffer::Buffer, layout::Rect, widgets::Widget, Terminal};

use crate::{config::Config, widgets::Connections};

pub struct App<'a> {
    connections: Connections<'a>,
}

impl<'a> App<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self {
            connections: Connections::new(config.conn.as_slice()),
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
            }
        }
    }

    fn draw<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> EventOutcome {
        let input = event.into();
        self.connections.on_input(input)
    }
}

impl Widget for &mut App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.connections.draw(area, buf);
    }
}
