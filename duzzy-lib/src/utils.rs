use std::{
    io::Read,
    path::{Path, PathBuf},
};

use anyhow::Context;
use serde::de::DeserializeOwned;

pub fn read_toml<T: Default + DeserializeOwned>(filepath: impl AsRef<Path>) -> anyhow::Result<T> {
    if let Ok(file) = std::fs::File::open(filepath) {
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

pub fn ensure_config_dir(app_name: &str) -> anyhow::Result<PathBuf> {
    // @note: linux & macos support only for now
    let mut dir_path: PathBuf = std::env::var("HOME")
        .with_context(|| "HOME env var")?
        .into();

    dir_path.push(".config");
    dir_path.push(app_name);

    if !dir_path.try_exists()? {
        std::fs::create_dir_all(&dir_path)?;
    }

    Ok(dir_path)
}
