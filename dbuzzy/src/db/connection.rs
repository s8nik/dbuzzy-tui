use deadpool_postgres::Runtime;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ConnectionConfig {
    pub name: Option<String>,
    pub host: String,
    pub port: u16,
    pub dbname: Option<String>,
    pub user: String,
    pub password: Option<String>,
}

impl std::fmt::Display for ConnectionConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name.as_ref().map_or_else(
            || format!("postgres://{}:@{}:{}", &self.user, &self.host, self.port),
            |n| n.to_owned(),
        );

        write!(f, "{}", name)
    }
}

// @todo: `postgres` feature
impl From<ConnectionConfig> for deadpool_postgres::Config {
    fn from(conf: ConnectionConfig) -> Self {
        Self {
            user: Some(conf.user),
            password: conf.password,
            dbname: conf.dbname,
            application_name: conf.name,
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
    pub async fn create(config: ConnectionConfig) -> super::DbResult<Self> {
        let pg_conf: deadpool_postgres::Config = config.into();
        let pool = pg_conf.create_pool(Some(Runtime::Tokio1), tokio_postgres::NoTls)?;

        Ok(Self { inner: pool })
    }

    pub async fn acquire(&self) -> super::DbResult<deadpool_postgres::Client> {
        let client = self.inner.get().await?;
        Ok(client)
    }
}
