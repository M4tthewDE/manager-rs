use std::{net::SocketAddr, path::PathBuf};

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
pub struct Config {
    pub address: SocketAddr,
    pub docker_compose_path: PathBuf,
    pub update_interval: u64,
}

impl Config {
    pub fn new(path: PathBuf) -> Result<Self> {
        let config: Config = toml::from_str(&std::fs::read_to_string(path)?)?;
        Ok(config)
    }
}
