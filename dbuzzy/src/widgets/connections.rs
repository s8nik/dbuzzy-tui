use duzzy_lib::{
    colors,
    event::{Event, Input},
    DuzzyWidget, EventOutcome,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::db::connection::{ConnectionConfig, PgPool};

pub struct ConnectionsWidget {
    state: ListState,
    configs: &'static [ConnectionConfig],
    pool: Option<PgPool>,
}

impl ConnectionsWidget {
    pub fn new(conns: &'static [ConnectionConfig]) -> Self {
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

    pub fn next_connection(&mut self) {
        let i = self.state.selected().map(|i| {
            if i >= self.configs.len() - 1 {
                0
            } else {
                i + 1
            }
        });

        self.state.select(i);
    }

    pub fn prev_connection(&mut self) {
        let i = self.state.selected().map(|i| {
            if i == 0 {
                self.configs.len() - 1
            } else {
                i - 1
            }
        });

        self.state.select(i);
    }

    pub fn select_connection(&mut self) {
        let Some(config) = self.state.selected().and_then(|i| self.configs.get(i)) else {
            return;
        };

        self.pool = match PgPool::create(config) {
            Ok(pool) => Some(pool),
            Err(_e) => {
                // @todo: call error widget?
                // and better logs
                return;
            }
        };
    }

    // @todo:
    #[allow(dead_code)]
    pub const fn pool(&self) -> Option<&PgPool> {
        self.pool.as_ref()
    }
}

impl DuzzyWidget for ConnectionsWidget {
    type Outcome = super::AppEventOutcome;

    fn input(&mut self, input: Input) -> Self::Outcome {
        let mut outcome = EventOutcome::Render;

        match input.event {
            Event::Char('q') | Event::Esc => outcome = EventOutcome::Exit,
            Event::Char('j') | Event::Down => self.next_connection(),
            Event::Char('k') | Event::Up => self.prev_connection(),
            Event::Char('l') | Event::Right | Event::Enter => self.select_connection(),
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
