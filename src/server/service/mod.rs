use compose::ComposeService;
use docker::DockerService;
use lib::proto::{
    compose_server::ComposeServer, docker_server::DockerServer, system_server::SystemServer,
};
use system::SystemService;

use crate::config::Config;

mod compose;
mod docker;
mod system;

pub fn docker() -> DockerServer<DockerService> {
    DockerServer::new(DockerService::default())
}

pub fn system() -> SystemServer<SystemService> {
    let service = SystemService::new();
    SystemServer::new(service)
}

pub fn compose(config: Config) -> ComposeServer<ComposeService> {
    ComposeServer::new(ComposeService::from(config))
}
