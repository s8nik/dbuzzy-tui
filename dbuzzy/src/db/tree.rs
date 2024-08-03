use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use anyhow::Context;
use deadpool_postgres::{Client, GenericClient};

const DEFAULT_INDENT: u8 = 4;

type TreeItemWeak = Weak<RefCell<TreeItem>>;
pub type TreeItemRef = Rc<RefCell<TreeItem>>;

#[derive(Debug, Default)]
pub struct DatabaseTree(Vec<TreeItemRef>);

impl AsRef<[TreeItemRef]> for DatabaseTree {
    fn as_ref(&self) -> &[TreeItemRef] {
        &self.0
    }
}

impl From<TreeItem> for TreeItemRef {
    fn from(item: TreeItem) -> Self {
        Self::new(RefCell::new(item))
    }
}

#[derive(Debug, PartialEq)]
pub struct TreeItem {
    pub indent: u8,
    pub name: String,
    pub kind: TreeItemKind,
}

impl TreeItem {
    pub const fn database(name: String) -> Self {
        Self {
            name,
            indent: 0,
            kind: TreeItemKind::Database { collapsed: false },
        }
    }

    pub const fn schema(name: String, database: TreeItemRef) -> Self {
        Self {
            name,
            indent: DEFAULT_INDENT,
            kind: TreeItemKind::Schema {
                database,
                collapsed: false,
            },
        }
    }

    pub const fn table(name: String, schema: TreeItemRef, database: TreeItemRef) -> Self {
        Self {
            name,
            indent: DEFAULT_INDENT * 2,
            kind: TreeItemKind::Table { schema, database },
        }
    }

    pub const fn is_database(&self) -> bool {
        matches!(self.kind, TreeItemKind::Database { .. })
    }

    pub const fn is_schema(&self) -> bool {
        matches!(self.kind, TreeItemKind::Schema { .. })
    }

    pub const fn is_table(&self) -> bool {
        matches!(self.kind, TreeItemKind::Table { .. })
    }

    pub const fn is_collapsed(&self) -> bool {
        match self.kind {
            TreeItemKind::Database { collapsed } => collapsed,
            TreeItemKind::Schema { collapsed, .. } => collapsed,
            TreeItemKind::Table { .. } => true,
        }
    }

    pub fn is_visible(&self) -> bool {
        match self.kind {
            TreeItemKind::Database { .. } => true,
            TreeItemKind::Schema { ref database, .. } => !database.borrow().is_collapsed(),
            TreeItemKind::Table {
                ref schema,
                ref database,
            } => !schema.borrow().is_collapsed() && !database.borrow().is_collapsed(),
        }
    }

    pub fn set_collapse(&mut self, collapse: bool) {
        match &mut self.kind {
            TreeItemKind::Database { collapsed } | TreeItemKind::Schema { collapsed, .. } => {
                *collapsed = collapse;
            }
            _ => (),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TreeItemKind {
    Database {
        collapsed: bool,
    },
    Schema {
        collapsed: bool,
        database: TreeItemRef,
    },
    Table {
        schema: TreeItemRef,
        database: TreeItemRef,
    },
}

impl DatabaseTree {
    pub async fn load(client: &Client) -> anyhow::Result<Self> {
        let stmt = client
            .prepare(
                r#"
                    SELECT
                        table_catalog AS database,
                        table_schema AS schema,
                        jsonb_agg(table_name) AS tables
                    FROM information_schema.tables
                    WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
                    GROUP BY table_catalog, table_schema
                    ORDER BY table_catalog
                "#,
            )
            .await?;

        let mut tree = vec![];
        let mut db = TreeItemWeak::default();

        let rows = client.query(&stmt, &[]).await?;

        for row in rows {
            let db_item = TreeItem::database(row.try_get("database")?).into();
            new_db_tree(&db_item, &mut db, &mut tree);

            let db_item = db.upgrade().with_context(|| "shouldn't be dropped")?;
            let schema_item = TreeItem::schema(row.try_get("schema")?, Rc::clone(&db_item)).into();

            tree.push(Rc::clone(&schema_item));

            let json_value: serde_json::Value = row.try_get("tables")?;
            let tables: Vec<String> = serde_json::from_value(json_value)?;

            for table in tables {
                let table_item =
                    TreeItem::table(table, Rc::clone(&schema_item), Rc::clone(&db_item));

                tree.push(table_item.into());
            }
        }

        Ok(Self(tree))
    }
}

fn new_db_tree(item: &TreeItemRef, db: &mut TreeItemWeak, tree: &mut Vec<TreeItemRef>) {
    fn inner(item: &TreeItemRef, db: &mut TreeItemWeak, tree: &mut Vec<TreeItemRef>) {
        *db = Rc::downgrade(item);
        tree.push(Rc::clone(item));
    }

    if let Some(db_ref) = db.upgrade() {
        if db_ref.borrow().name != item.borrow().name {
            inner(item, db, tree);
        }
    } else {
        inner(item, db, tree);
    }
}
