use deadpool_postgres::Runtime;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ConnCfg {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub db: Option<String>,
    pub user: String,
    pub password: Option<String>,
}

// @todo: `postgres` feature
impl From<ConnCfg> for deadpool_postgres::Config {
    fn from(conf: ConnCfg) -> Self {
        deadpool_postgres::Config {
            user: Some(conf.user),
            password: conf.password,
            dbname: conf.db,
            application_name: Some(conf.name),
            host: Some(conf.host),
            port: Some(conf.port),
            connect_timeout: Some(std::time::Duration::from_secs(5)),
            keepalives: Some(true),
            ..Default::default()
        }
    }
}
pub struct Pool {
    inner: deadpool_postgres::Pool,
}

impl Pool {
    #[must_use]
    pub async fn create(config: ConnCfg) -> super::DbResult<Self> {
        let pg_conf: deadpool_postgres::Config = config.into();
        let pool = pg_conf.create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)?;

        Ok(Self { inner: pool })
    }

    pub async fn acquire(&self) -> super::DbResult<deadpool_postgres::Client> {
        let client = self.inner.get().await?;
        Ok(client)
    }
}
