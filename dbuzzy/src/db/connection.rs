use deadpool_postgres::Runtime;

use super::config::ConnConfig;

pub struct Connection {
    pool: deadpool_postgres::Pool,
}

impl Connection {
    pub async fn new(config: ConnConfig) -> super::DbResult<Self> {
        let pg_conf: deadpool_postgres::Config = config.into();
        let pool = pg_conf.create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)?;

        Ok(Self { pool })
    }

    pub fn execute(&self) -> super::DbResult<()> {
        todo!()
    }

    pub fn databases(&self) -> super::DbResult<()> {
        todo!()
    }

    pub fn tables(&self) -> super::DbResult<()> {
        todo!()
    }

    pub fn records(&self) -> super::DbResult<()> {
        todo!()
    }

    pub fn columns(&self) -> super::DbResult<()> {
        todo!()
    }
}
