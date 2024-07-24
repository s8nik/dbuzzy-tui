use std::collections::HashMap;

use deadpool_postgres::{Client, GenericClient};
use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct DatabaseTree(HashMap<String, TreeItem>);

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TreeItem {
    Schemas(HashMap<String, TreeItem>),
    Tables(Vec<String>),
}

impl AsRef<HashMap<String, TreeItem>> for DatabaseTree {
    fn as_ref(&self) -> &HashMap<String, TreeItem> {
        &self.0
    }
}

pub async fn database_tree(client: &Client) -> anyhow::Result<DatabaseTree> {
    let stmt = client
        .prepare(
            r#"
                WITH schemas AS (
                    SELECT
                        table_catalog as database,
                        table_schema as schema,
                        jsonb_agg(table_name) AS tables
                    FROM information_schema.tables
                    GROUP BY table_catalog, table_schema
                )
                SELECT
                    datname as name,
                    jsonb_object_agg(schema, tables) AS schemas
                FROM pg_database
                INNER JOIN schemas on database = datname
                GROUP BY datname
            "#,
        )
        .await?;

    let rows = client.query(&stmt, &[]).await?;

    let mut map = HashMap::new();
    for row in rows {
        let name: String = row.try_get("name")?;
        let schemas: serde_json::Value = row.try_get("schemas")?;
        let tree_item = serde_json::from_value(schemas)?;
        map.insert(name, tree_item);
    }

    Ok(DatabaseTree(map))
}
