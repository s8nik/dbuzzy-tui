use crossterm::event::{Event, KeyCode, KeyEventKind};
use duzzy_lib::{colors, DrawableStateful, EventOutcome, OnEvent};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

use crate::db::connection::ConnCfg;

pub struct ConnWidget {
    items: Vec<ConnCfg>,
    state: ListState,
}

impl ConnWidget {
    pub fn new(conns: Vec<ConnCfg>) -> Self {
        let mut state = ListState::default();

        if !conns.is_empty() {
            state.select(Some(0));
        }

        Self {
            items: conns,
            state,
        }
    }

    pub fn next_conn(&mut self) {
        let i = self
            .state
            .selected()
            .map(|i| if i >= self.items.len() - 1 { 0 } else { i + 1 });

        self.state.select(i);
    }

    pub fn prev_conn(&mut self) {
        let i = self
            .state
            .selected()
            .map(|i| if i == 0 { self.items.len() - 1 } else { i - 1 });

        self.state.select(i);
    }

    pub fn selected_conn(&self) -> Option<&ConnCfg> {
        self.state.selected().and_then(|i| self.items.get(i))
    }
}

impl DrawableStateful for ConnWidget {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let vertical = Layout::vertical([Constraint::Min(0), Constraint::Length(2)]);

        let [conn_area, info_area] = vertical.areas(area);

        Paragraph::new("\nUse j/k to move. Enter to select connection")
            .centered()
            .render(info_area, buf);

        let items = self
            .items
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

impl OnEvent for ConnWidget {
    fn on_event(&mut self, event: Event) -> EventOutcome {
        let mut outcome = EventOutcome::Render;

        let Event::Key(key) = event else {
            return EventOutcome::Ignore;
        };

        if key.kind == KeyEventKind::Press {
            use KeyCode::*;

            match key.code {
                Char('q') | Esc => outcome = EventOutcome::Exit,
                Char('j') | Down => self.next_conn(),
                Char('k') | Up => self.prev_conn(),
                Char('l') | Right | Enter => todo!(),
                _ => outcome = EventOutcome::Ignore,
            }
        }

        outcome
    }
}
