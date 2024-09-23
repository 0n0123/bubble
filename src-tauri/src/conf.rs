use std::{fs, path::Path};

use anyhow::{anyhow, Result};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub server: Vec<String>,
}

impl Config {
    pub fn read(path: &Path) -> Result<Config> {
        let config = fs::read_to_string(path)?;
        toml::from_str::<Config>(&config).map_err(|_| anyhow!("Failed to parse config."))
    }
}
