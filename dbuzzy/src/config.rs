use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub conn: Vec<ConnConfig>,
}

impl Config {
    pub fn from_toml() -> anyhow::Result<Self> {
        let app = std::env!("CARGO_PKG_NAME");
        duzzy_lib::config_toml(&app, "config")
    }
}

#[derive(Debug, Deserialize)]
pub struct ConnConfig {
    name: String,
    host: String,
    port: u16,
    db: Option<String>,
    user: String,
    password: Option<String>,
}

impl From<ConnConfig> for deadpool_postgres::Config {
    fn from(conf: ConnConfig) -> Self {
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
