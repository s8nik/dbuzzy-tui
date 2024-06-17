use duzzy_lib::{colors, DrawableStateful};
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
        Self {
            items: conns,
            state: ListState::default(),
        }
    }

    pub fn next_conn(&mut self) {
        let i = self
            .state
            .selected()
            .map(|i| if i > self.items.len() - 1 { 0 } else { i + 1 });

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
        let vertical = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(0),
            Constraint::Length(2),
        ]);

        let [header_area, rest_area, footer_area] = vertical.areas(area);

        Paragraph::new("Connections:")
            .bold()
            .centered()
            .render(header_area, buf);

        Paragraph::new("\nUse j/k to move. Enter to select connection")
            .centered()
            .render(footer_area, buf);

        let items = self
            .items
            .iter()
            .map(|conn| {
                let line = Line::styled(conn.to_string(), colors::ALOE_GREEN);
                ListItem::new(line).bg(colors::RICH_BLACK)
            })
            .collect::<Vec<_>>();

        let connections = List::new(items)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().bg(colors::RICH_BLACK))
            .style(Style::default());

        StatefulWidget::render(connections, rest_area, buf, &mut self.state);
    }
}
