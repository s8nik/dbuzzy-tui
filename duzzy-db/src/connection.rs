use deadpool_postgres::Runtime;

use super::config::ConnConfig;

pub struct Connection {
    pool: deadpool_postgres::Pool,
}

impl Connection {
    pub async fn new(config: ConnConfig) -> anyhow::Result<Self> {
        let pg_conf: deadpool_postgres::Config = config.into();
        let pool = pg_conf.create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)?;

        Ok(Self { pool })
    }
}
