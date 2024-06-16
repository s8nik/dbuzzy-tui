use std::{io::Read, path::PathBuf};

use anyhow::Context;
use serde::de::DeserializeOwned;

pub fn config_toml<T: Default + DeserializeOwned>(
    app_name: &str,
    cfg_name: &str,
) -> anyhow::Result<T> {
    // @note: linux & macos support only for now
    let mut config_path: PathBuf = std::env::var("$HOME")
        .with_context(|| "HOME env var")?
        .into();

    config_path.push(".config");
    config_path.push(app_name);

    if !config_path.try_exists()? {
        std::fs::create_dir_all(&config_path)?;
    }

    config_path.push(cfg_name);
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

    Ok(T::default())
}
