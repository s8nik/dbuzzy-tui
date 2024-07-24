use std::collections::HashMap;

use duzzy_lib::DuzzyWidget;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Widget},
};

use crate::db::{
    connection::PgPool,
    queries::{DatabaseTree, TreeItem},
};

#[derive(Default)]
pub struct DatabasesWidget {
    tree: DatabaseTree,
}

impl DatabasesWidget {
    // @todo:
    #[allow(dead_code)]
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
        let new_item =
            |i: usize, content: &str| ListItem::new(format!("{}└── {content}", " ".repeat(i)));

        for (parent, child) in tree {
            items.push(new_item(indent, parent.as_str()));

            match child {
                TreeItem::Schemas(m) => return Self::draw_tree(indent + 1, items, m),
                TreeItem::Tables(v) => v
                    .iter()
                    .for_each(|table| items.push(new_item(indent + 1, table.as_str()))),
            }
        }

        items
    }
}

impl DuzzyWidget for DatabasesWidget {
    type Outcome = super::AppEventOutcome;

    fn input(&mut self, _input: duzzy_lib::event::Input) -> Self::Outcome {
        todo!()
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
                .borders(Borders::ALL),
        );

        Widget::render(tree, tree_area, buf);
    }
}
