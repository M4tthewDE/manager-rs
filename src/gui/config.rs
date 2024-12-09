use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub update_interval: u64,
    pub profiling: bool,
}

impl Config {
    pub fn new(path: PathBuf) -> Result<Self> {
        let config: Config = toml::from_str(&std::fs::read_to_string(path)?)?;

        Ok(config)
    }
}
