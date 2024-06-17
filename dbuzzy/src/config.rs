use serde::Deserialize;

use crate::db::connection::ConnCfg;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub conn: Vec<ConnCfg>,
}

impl Config {
    pub fn from_toml() -> anyhow::Result<Self> {
        let app = std::env!("CARGO_PKG_NAME");
        duzzy_lib::config_toml(&app, "config")
    }
}

#[cfg(test)]
mod tests {}
