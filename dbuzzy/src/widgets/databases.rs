use std::collections::HashMap;

use duzzy_lib::{colors, event::Event, DuzzyWidget, EventOutcome};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::Stylize,
    text::Line,
    widgets::{Block, Borders, List, ListItem, Widget},
};

use crate::db::{
    connection::PgPool,
    queries::{DatabaseTree, TreeItem},
};

const INDENT_OFFSET: usize = 4;

#[derive(Default)]
pub struct DatabaseTreeWidget {
    tree: DatabaseTree,
}

impl DatabaseTreeWidget {
    pub async fn new(pool: &PgPool) -> anyhow::Result<Self> {
        let mut widget = Self::default();
        widget.update(pool).await?;
        Ok(widget)
    }

    pub async fn update(&mut self, pool: &PgPool) -> anyhow::Result<()> {
        let conn = pool.acquire().await?;
        self.tree = crate::db::queries::database_tree(&conn).await?;
        Ok(())
    }

    fn draw_tree<'a>(
        indent: usize,
        mut items: Vec<ListItem<'a>>,
        tree: &'a HashMap<String, TreeItem>,
    ) -> Vec<ListItem<'a>> {
        let new_item = |new_indent: usize, content: &str, is_last: bool| {
            let sign = if is_last { "└──" } else { "├──" };
            ListItem::new(Line::styled(
                format!("{}{sign} {content}", " ".repeat(new_indent)),
                colors::LIGHT_GOLDENROD_YELLOW,
            ))
        };

        for (parent, children) in tree.iter() {
            items.push(new_item(indent, parent.as_str(), true));

            match children {
                TreeItem::Schemas(m) => items = Self::draw_tree(indent + INDENT_OFFSET, items, m),
                TreeItem::Tables(v) => v.iter().enumerate().for_each(|(i, table)| {
                    items.push(new_item(
                        indent + INDENT_OFFSET,
                        table.as_str(),
                        i == v.len(),
                    ));
                }),
            }
        }

        items
    }
}

impl DuzzyWidget for DatabaseTreeWidget {
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

        let items = Self::draw_tree(0, vec![], self.tree.as_ref());
        let tree = List::new(items).block(
            Block::default()
                .title("Database Tree")
                .borders(Borders::ALL)
                .fg(colors::ENERGY_YELLOW),
        );

        Widget::render(tree, tree_area, buf);
    }
}
