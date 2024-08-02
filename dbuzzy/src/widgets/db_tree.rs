use duzzy_lib::{
    colors, duzzy_lib_derive::DuzzyListImpl, event::Event, DuzzyList, DuzzyListState, DuzzyWidget,
    EventOutcome,
};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget},
};

use crate::db::{tree::DatabaseTree, PgPool};

#[derive(DuzzyListImpl)]
pub struct DbTreeWidget {
    state: ListState,
    inner: DatabaseTree,
    pool: PgPool,
}

impl DbTreeWidget {
    pub fn new(pool: PgPool) -> Self {
        Self {
            state: ListState::default(),
            inner: DatabaseTree::default(),
            pool,
        }
    }

    pub async fn update(&mut self) -> anyhow::Result<()> {
        let conn = self.pool.acquire().await?;
        self.inner = DatabaseTree::load(&conn).await?;

        if !self.inner.as_ref().is_empty() {
            self.state.select(Some(0));
        }

        Ok(())
    }
}

impl DuzzyListState for DbTreeWidget {
    fn state(&mut self) -> &mut ListState {
        &mut self.state
    }

    fn length(&self) -> usize {
        self.inner
            .as_ref()
            .iter()
            .filter(|x| x.is_visible())
            .count()
    }
}

const OPEN_INDENT_ICON: &str = "├──";
const CLOSE_INDENT_ICON: &str = "└──";

impl DuzzyWidget for DbTreeWidget {
    type Outcome = super::AppEventOutcome;

    fn input(&mut self, input: duzzy_lib::event::Input) -> Self::Outcome {
        let mut outcome = EventOutcome::Render;

        match input.event {
            Event::Char('q') | Event::Esc => outcome = EventOutcome::Exit,
            Event::Char('j') | Event::Down => self.next(),
            Event::Char('k') | Event::Up => self.prev(),
            _ => outcome = EventOutcome::Ignore,
        };

        outcome.into()
    }

    fn render(&mut self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let tree_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(100).as_ref()])
            .split(area)[0];

        let mut items = vec![];
        let tree_list = self
            .inner
            .as_ref()
            .iter()
            .filter(|x| x.is_visible())
            .enumerate()
            .collect::<Vec<_>>();

        let tree_len = tree_list.len() - 1;

        for (i, tree_item) in tree_list {
            let is_close = i == tree_len || !tree_item.is_collapsed();

            let indent_icon = if is_close {
                CLOSE_INDENT_ICON
            } else {
                OPEN_INDENT_ICON
            };

            let item = ListItem::new(Line::styled(
                format!(
                    "{}{indent_icon} {}",
                    " ".repeat(tree_item.indent as usize),
                    tree_item.name
                ),
                colors::LIGHT_GOLDENROD_YELLOW,
            ));

            items.push(item);
        }

        let tree = List::new(items)
            .block(
                Block::default()
                    .title("Database Tree")
                    .borders(Borders::ALL)
                    .fg(colors::ENERGY_YELLOW),
            )
            .highlight_symbol(">")
            .highlight_style(Style::default().bg(colors::ALOE_GREEN));

        StatefulWidget::render(tree, tree_area, buf, &mut self.state);
    }
}
