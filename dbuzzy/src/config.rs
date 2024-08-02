use serde::Deserialize;

use crate::db::ConnectionConfig;

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub conn: Vec<ConnectionConfig>,
}

impl Config {
    pub fn from_toml() -> anyhow::Result<Self> {
        let mut path = duzzy_lib::ensure_config_dir(std::env!("CARGO_PKG_NAME"))?;
        path.push("config");
        path.set_extension("toml");

        duzzy_lib::read_toml(path)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::Config;

    #[test]
    fn test_from_toml() -> anyhow::Result<()> {
        let content = r#"
            [[conn]]
            user = "foo"
            host = "localhost"
            port = 5432
            dbname = "bar"
        "#;

        let mut filepath = duzzy_lib::ensure_config_dir(std::env!("CARGO_PKG_NAME"))?;
        filepath.push("test");
        filepath.set_extension("toml");

        {
            let file = std::fs::File::create(&filepath)?;
            let mut writer = std::io::BufWriter::new(file);
            writer.write_all(content.as_bytes())?;
        }

        let cfg: Config = duzzy_lib::read_toml(&filepath)?;

        assert_eq!(cfg.conn.len(), 1);
        assert_eq!(&cfg.conn[0].host, "localhost");
        assert_eq!(&cfg.conn[0].user, "foo");
        assert_eq!(cfg.conn[0].port, 5432);
        assert_eq!(cfg.conn[0].dbname.as_deref(), Some("bar"));

        std::fs::remove_file(filepath)?;

        Ok(())
    }
}
