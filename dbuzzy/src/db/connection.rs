use deadpool_postgres::{GenericClient, Runtime};

use crate::config::ConnConfig;

use super::types::{ColumnRow, DatabaseRow, TableRow};

pub struct Pool {
    inner: deadpool_postgres::Pool,
}

impl Pool {
    #[must_use]
    pub async fn create(config: ConnConfig) -> super::DbResult<Self> {
        let pg_conf: deadpool_postgres::Config = config.into();
        let pool = pg_conf.create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)?;

        Ok(Self { inner: pool })
    }

    pub async fn acquire(&self) -> super::DbResult<deadpool_postgres::Client> {
        let client = self.inner.get().await?;
        Ok(client)
    }
}

pub async fn execute(client: &deadpool_postgres::Client) -> super::DbResult<()> {
    todo!()
}

pub async fn databases(client: &deadpool_postgres::Client) -> super::DbResult<Vec<DatabaseRow>> {
    let stmt = client.prepare("SELECT datname FROM pg_database").await?;
    let rows = client.query(&stmt, &[]).await?;
    Ok(rows.into_iter().map(Into::<DatabaseRow>::into).collect())
}

pub async fn tables(
    client: &deadpool_postgres::Client,
    dbname: &DatabaseRow,
) -> super::DbResult<Vec<TableRow>> {
    let stmt = client
        .prepare(
            r#"
            SELECT
                table_name,
                table_schema
            FROM information_schema.tables
            WHERE table_catalog = $1
            "#,
        )
        .await?;

    let rows = client.query(&stmt, &[&dbname.as_ref()]).await?;
    Ok(rows.into_iter().map(Into::<TableRow>::into).collect())
}

pub async fn columns(
    client: &deadpool_postgres::Client,
    dbname: &DatabaseRow,
    table: &TableRow,
) -> super::DbResult<Vec<ColumnRow>> {
    let stmt = client
        .prepare(
            r#"
            SELECT
                column_name,
                data_type,
                is_nullable,
                column_default
            FROM information_schema.columns
            WHERE table_catalog = $1
                AND table_schema = $2
                AND table_name = $3
            "#,
        )
        .await?;

    let rows = client
        .query(&stmt, &[&dbname.as_ref(), &table.schema, &table.name])
        .await?;

    Ok(rows.into_iter().map(Into::<ColumnRow>::into).collect())
}
