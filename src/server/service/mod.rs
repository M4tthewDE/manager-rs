use std::sync::{Arc, Mutex};

use crate::{
    proto::{
        compose_server::ComposeServer, docker_server::DockerServer, system_server::SystemServer,
    },
    subscriber::LogRelay,
};
use compose::ComposeService;
use docker::DockerService;
use system::SystemService;

use crate::config::Config;

mod compose;
mod docker;
mod system;

pub fn docker() -> DockerServer<DockerService> {
    DockerServer::new(DockerService::default())
}

pub fn system(config: Config, log_relay: Arc<Mutex<LogRelay>>) -> SystemServer<SystemService> {
    SystemServer::new(SystemService::new(config, log_relay))
}

pub fn compose(config: Config) -> ComposeServer<ComposeService> {
    ComposeServer::new(ComposeService::from(config))
}
