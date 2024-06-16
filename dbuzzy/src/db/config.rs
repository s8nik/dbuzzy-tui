use std::{io::Read, path::PathBuf};

use anyhow::Context;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ConnectionConfig {
    name: String,
    host: String,
    port: u16,
    db: Option<String>,
    user: String,
    password: Option<String>,
}

impl ConnectionConfig {
    // @todo: should be moved as it is general purpose
    pub fn from_file() -> anyhow::Result<Self> {
        // @note: linux & macos support only for now
        let mut config_path: PathBuf = std::env::var("$HOME")
            .with_context(|| "HOME env var")?
            .into();

        config_path.push(".config/dbuzzy");

        if !config_path.try_exists()? {
            std::fs::create_dir_all(&config_path)?;
        }

        config_path.push(std::env!("CARGO_PKG_NAME"));
        config_path.set_extension("toml");

        if let Ok(file) = std::fs::File::open(config_path) {
            let mut reader = std::io::BufReader::new(file);
            let mut raw_config = String::new();
            reader.read_to_string(&mut raw_config)?;

            match toml::from_str(&raw_config) {
                Ok(c) => return Ok(c),
                Err(e) => anyhow::bail!("fail to parse config file: {e}"),
            }
        }

        Ok(ConnConfigBuilder::new().build())
    }
}

#[derive(Default)]
pub struct ConnConfigBuilder {
    name: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    db: Option<String>,
    user: Option<String>,
    password: Option<String>,
}

impl ConnConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn host(mut self, host: String) -> Self {
        self.host = Some(host);
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = Some(port);
        self
    }

    pub fn db(mut self, db: String) -> Self {
        self.db = Some(db);
        self
    }

    pub fn user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    pub fn password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    pub fn build(self) -> ConnectionConfig {
        self.into()
    }
}

impl From<ConnConfigBuilder> for ConnectionConfig {
    fn from(builder: ConnConfigBuilder) -> Self {
        let host = builder.host.unwrap_or("localhost".to_owned());
        let port = builder.port.unwrap_or(5432);
        let user = builder.user.unwrap_or("postgres".to_owned());
        let name = builder
            .name
            .unwrap_or_else(|| format!("{}@{}:{}", &user, &host, &port));

        Self {
            name,
            host,
            port,
            user,
            db: builder.db,
            password: builder.password,
        }
    }
}

impl From<ConnectionConfig> for deadpool_postgres::Config {
    fn from(conf: ConnectionConfig) -> Self {
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

#[cfg(test)]
mod tests {}
