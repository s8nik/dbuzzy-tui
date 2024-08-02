use std::sync::{Arc, Weak};

use anyhow::Context;
use deadpool_postgres::{Client, GenericClient};

const DEFAULT_INDENT: u8 = 4;

#[derive(Debug, Default)]
pub struct DatabaseTree(Vec<Arc<TreeItem>>);

impl AsRef<[Arc<TreeItem>]> for DatabaseTree {
    fn as_ref(&self) -> &[Arc<TreeItem>] {
        &self.0
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

    pub const fn schema(name: String, database: Arc<Self>) -> Self {
        Self {
            name,
            indent: DEFAULT_INDENT,
            kind: TreeItemKind::Schema {
                database,
                collapsed: false,
            },
        }
    }

    pub const fn table(name: String, schema: Arc<Self>, database: Arc<Self>) -> Self {
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
            TreeItemKind::Schema { ref database, .. } => !database.is_collapsed(),
            TreeItemKind::Table {
                ref schema,
                ref database,
            } => !schema.is_collapsed() || !database.is_collapsed(),
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
        database: Arc<TreeItem>,
    },
    Table {
        schema: Arc<TreeItem>,
        database: Arc<TreeItem>,
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
        let mut db = Weak::<TreeItem>::default();

        let rows = client.query(&stmt, &[]).await?;

        for row in rows {
            let db_item = Arc::new(TreeItem::database(row.try_get("database")?));
            new_db_tree(&db_item, &mut db, &mut tree);

            let db_item = db.upgrade().with_context(|| "shouldn't be dropped")?;

            let schema_item = Arc::new(TreeItem::schema(
                row.try_get("schema")?,
                Arc::clone(&db_item),
            ));

            tree.push(Arc::clone(&schema_item));

            let json_value: serde_json::Value = row.try_get("tables")?;
            let tables: Vec<String> = serde_json::from_value(json_value)?;

            for table in tables {
                let table_item = Arc::new(TreeItem::table(
                    table,
                    Arc::clone(&schema_item),
                    Arc::clone(&db_item),
                ));

                tree.push(table_item);
            }
        }

        Ok(Self(tree))
    }
}

fn new_db_tree(item: &Arc<TreeItem>, db: &mut Weak<TreeItem>, tree: &mut Vec<Arc<TreeItem>>) {
    fn inner(item: &Arc<TreeItem>, db: &mut Weak<TreeItem>, tree: &mut Vec<Arc<TreeItem>>) {
        *db = Arc::downgrade(item);
        tree.push(Arc::clone(item));
    }

    if let Some(db_ref) = db.upgrade() {
        if db_ref.name != item.name {
            inner(item, db, tree);
        }
    } else {
        inner(item, db, tree);
    }
}
