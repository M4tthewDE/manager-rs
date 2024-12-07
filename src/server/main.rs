use docker_proto::{
    docker_server::{Docker, DockerServer},
    Container, ContainerListReply, Empty, Port,
};
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

mod docker_proto {
    tonic::include_proto!("docker");
}

mod docker;
mod memory;

#[derive(Debug, Default)]
pub struct DockerService {}

#[tonic::async_trait]
impl Docker for DockerService {
    async fn list_containers(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<ContainerListReply>, Status> {
        info!("REQUEST: {request:?}");

        let containers = docker::list_containers()
            .await
            .map_err(|e| Status::from_error(e.into()))?;

        let container_list: Vec<Container> = containers
            .iter()
            .map(|c| Container {
                names: c.names.clone(),
                image: c.image.clone(),
                command: c.command.clone(),
                created: c.created,
                ports: c
                    .ports
                    .iter()
                    .map(|p| Port {
                        private_port: p.private_port,
                        public_port: p.public_port,
                        port_type: p.port_type.clone(),
                    })
                    .collect(),
                status: c.status.clone(),
            })
            .collect();

        Ok(Response::new(ContainerListReply { container_list }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let addr = "[::1]:50051".parse()?;
    let docker = DockerService::default();

    Server::builder()
        .add_service(DockerServer::new(docker))
        .add_service(memory::service())
        .serve(addr)
        .await?;

    Ok(())
}
