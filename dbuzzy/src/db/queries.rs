use std::collections::HashMap;

use deadpool_postgres::{Client, GenericClient};
use tokio_postgres::Row;

#[derive(Debug)]
pub struct DatabaseEntity {
    name: String,
    schemas: HashMap<String, Vec<String>>,
}

impl TryFrom<Row> for DatabaseEntity {
    type Error = anyhow::Error;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        let schemas: serde_json::Value = row.try_get("schemas")?;

        Ok(Self {
            name: row.try_get("name")?,
            schemas: serde_json::from_value(schemas)?,
        })
    }
}

pub async fn database_tree(client: &Client) -> anyhow::Result<Vec<DatabaseEntity>> {
    let stmt = client
        .prepare(
            r#"
                with schemas as (
                	select
                		table_catalog as database,
                		table_schema as schema,
                		jsonb_agg(table_name) as tables
                	from information_schema.tables
                	group by table_catalog, table_schema
                )
                select
                	datname as name,
                	jsonb_object_agg(schema, tables) as schemas
                from pg_database
                inner join schemas on database = datname
                group by datname
            "#,
        )
        .await?;

    let rows = client.query(&stmt, &[]).await?;
    rows.into_iter()
        .map(TryInto::<DatabaseEntity>::try_into)
        .collect::<Result<Vec<_>, _>>()
}
