use anyhow::Result;
use container::Container;
use version::Version;

use super::{State, StateChangeMessage};

pub mod container;
pub mod version;

#[derive(Default)]
pub struct DockerState {
    pub containers: Vec<Container>,
    pub version: Version,
}

pub async fn update(server_address: String) -> Result<StateChangeMessage> {
    let containers = container::get_containers(server_address.clone()).await?;
    let version = version::get_version(server_address).await?;

    Ok(Box::new(move |state: &mut State| {
        state.docker_state.containers = containers;
        state.docker_state.version = version;
    }))
}
