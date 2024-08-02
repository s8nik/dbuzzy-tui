use duzzy_lib::{
    colors,
    duzzy_lib_derive::DuzzyListImpl,
    event::{Event, Input},
    DuzzyList, DuzzyListState, DuzzyWidget, EventOutcome,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::db::{ConnConfig, PgPool};

#[derive(DuzzyListImpl)]
pub struct ConnListWidget {
    state: ListState,
    configs: &'static [ConnConfig],
    pool: Option<PgPool>,
}

impl ConnListWidget {
    pub fn new(conns: &'static [ConnConfig]) -> Self {
        let mut state = ListState::default();

        if !conns.is_empty() {
            state.select(Some(0));
        }

        Self {
            configs: conns,
            state,
            pool: None,
        }
    }

    fn select_connection(&mut self) {
        if let Some(config) = self.state.selected().and_then(|i| self.configs.get(i)) {
            self.pool = PgPool::create(config).ok();
        };
    }

    pub const fn pool(&self) -> Option<&PgPool> {
        self.pool.as_ref()
    }
}

impl DuzzyListState for ConnListWidget {
    fn state(&mut self) -> &mut ListState {
        &mut self.state
    }

    fn length(&self) -> usize {
        self.configs.len()
    }
}

impl DuzzyWidget for ConnListWidget {
    type Outcome = super::AppEventOutcome;

    fn input(&mut self, input: Input) -> Self::Outcome {
        let mut outcome = EventOutcome::Render;

        match input.event {
            Event::Char('q') | Event::Esc => outcome = EventOutcome::Exit,
            Event::Char('j') | Event::Down => self.next(),
            Event::Char('k') | Event::Up => self.prev(),
            Event::Char('l') | Event::Right | Event::Enter => {
                self.select_connection();
                if let Some(pool) = self.pool().cloned() {
                    return super::AppEventOutcome::Apply(super::AppWidgetData::Connection(pool));
                }
            }
            _ => outcome = EventOutcome::Ignore,
        }

        outcome.into()
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([Constraint::Min(0), Constraint::Length(2)]);

        let [conn_area, info_area] = vertical.areas(area);

        Paragraph::new("\nUse j/k to move. Enter to select connection")
            .centered()
            .render(info_area, buf);

        let items = self
            .configs
            .iter()
            .map(|conn| {
                ListItem::new(Line::styled(
                    conn.to_string(),
                    colors::LIGHT_GOLDENROD_YELLOW,
                ))
            })
            .collect::<Vec<_>>();

        let connections = List::new(items)
            .block(
                Block::default()
                    .title("Connections:")
                    .borders(Borders::ALL)
                    .fg(colors::ENERGY_YELLOW),
            )
            .highlight_symbol(">")
            .highlight_style(Style::default().bg(colors::ALOE_GREEN));

        StatefulWidget::render(connections, conn_area, buf, &mut self.state);
    }
}
