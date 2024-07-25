use std::{collections::HashMap, time::Duration};

use crossterm::event::{Event, EventStream};
use duzzy_editor::Editor;
use duzzy_lib::{DuzzyWidget, EventOutcome};
use futures_util::StreamExt;
use ratatui::{backend::Backend, buffer::Buffer, layout::Rect, widgets::Widget, Terminal};

use crate::{
    config::Config,
    widgets::{
        AppEventOutcome, AppWidgetData, AppWidgetName, ConnectionsWidget, DatabaseTreeWidget,
    },
};

pub struct App {
    focus: AppWidgetName,
    editor: Box<Editor>,
    widgets: HashMap<AppWidgetName, Box<dyn DuzzyWidget<Outcome = AppEventOutcome>>>,
}

impl App {
    pub fn new(config: &'static Config) -> Self {
        let mut widgets: HashMap<AppWidgetName, Box<dyn DuzzyWidget<Outcome = AppEventOutcome>>> =
            HashMap::new();

        widgets.insert(
            AppWidgetName::Connections,
            Box::new(ConnectionsWidget::new(config.conn.as_slice())),
        );

        Self {
            widgets,
            editor: Box::new(Editor::new_scratch()),
            focus: AppWidgetName::Connections,
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
                AppEventOutcome::Focus(name) => self.focus = name,
                // @todo: show error widget
                #[allow(clippy::redundant_pattern_matching)]
                AppEventOutcome::Apply(data) => {
                    if let Err(_) = self.apply(data).await {
                        // @todo: error widget
                    }
                    self.draw(terminal)?;
                }
            }
        }
    }

    fn draw<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> anyhow::Result<()> {
        terminal.draw(|f| f.render_widget(self, f.size()))?;
        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> AppEventOutcome {
        let input = event.into();

        if self.focus == AppWidgetName::Editor {
            return self.editor.input(input).into();
        }

        self.focused().input(input)
    }

    async fn apply(&mut self, data: AppWidgetData) -> anyhow::Result<()> {
        match data {
            AppWidgetData::Connection(pool) => {
                self.widgets.insert(
                    AppWidgetName::DatabaseTree,
                    Box::new(DatabaseTreeWidget::new(&pool).await?),
                );

                self.focus = AppWidgetName::DatabaseTree;
            }
        };

        Ok(())
    }

    fn focused(&mut self) -> &mut Box<dyn DuzzyWidget<Outcome = AppEventOutcome>> {
        self.widgets.get_mut(&self.focus).expect("should focus")
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // @note: draw widgets based on currently focused one
        self.focused().render(area, buf);
    }
}
