use std::path::PathBuf;

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub update_interval: u64,
    pub profiling: bool,
    pub server_address: String,
    pub docker_compose_path: PathBuf,
}

impl Config {
    pub fn new(path: PathBuf) -> Result<Self> {
        let config: Config = toml::from_str(&std::fs::read_to_string(path)?)?;

        Ok(config)
    }
}
