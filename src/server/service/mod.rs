use crate::proto::{
    compose_server::ComposeServer, docker_server::DockerServer, system_server::SystemServer,
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

pub fn system(config: Config) -> SystemServer<SystemService> {
    SystemServer::new(SystemService::new(config))
}

pub fn compose(config: Config) -> ComposeServer<ComposeService> {
    ComposeServer::new(ComposeService::from(config))
}
