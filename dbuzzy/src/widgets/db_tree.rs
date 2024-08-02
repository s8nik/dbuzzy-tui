use duzzy_lib::{colors, event::Event, DuzzyWidget, EventOutcome};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, List, ListItem, Widget},
};

use crate::db::{tree::DatabaseTree, PgPool};

#[derive(Default)]
pub struct DbTreeWidget {
    inner: DatabaseTree,
}

impl DbTreeWidget {
    pub async fn new(pool: &PgPool) -> anyhow::Result<Self> {
        let mut widget = Self::default();
        widget.update(pool).await?;
        Ok(widget)
    }

    pub async fn update(&mut self, pool: &PgPool) -> anyhow::Result<()> {
        let conn = pool.acquire().await?;
        self.inner = DatabaseTree::load(&conn).await?;
        Ok(())
    }
}

const OPEN_INDENT_ICON: &str = "├──";
const CLOSE_INDENT_ICON: &str = "└──";

impl DuzzyWidget for DbTreeWidget {
    type Outcome = super::AppEventOutcome;

    fn input(&mut self, input: duzzy_lib::event::Input) -> Self::Outcome {
        let outcome = match input.event {
            Event::Char('q') | Event::Esc => EventOutcome::Exit,
            _ => EventOutcome::Ignore,
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

        let tree = List::new(items).block(
            Block::default()
                .title("Database Tree")
                .borders(Borders::ALL)
                .fg(colors::ENERGY_YELLOW),
        );

        Widget::render(tree, tree_area, buf);
    }
}
