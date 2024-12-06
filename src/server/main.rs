use docker_proto::{
    docker_server::{Docker, DockerServer},
    Container, ContainerListReply, Empty,
};
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

mod docker_proto {
    tonic::include_proto!("docker");
}

mod docker;

#[derive(Debug, Default)]
pub struct DockerService {}

#[tonic::async_trait]
impl Docker for DockerService {
    async fn list_containers(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<ContainerListReply>, Status> {
        info!("REQUEST: {request:?}");

        let containers = docker::list_containers().await.unwrap();
        let container_list: Vec<Container> = containers
            .iter()
            .map(|c| Container {
                names: c.names.clone(),
                image: c.image.clone(),
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
        .serve(addr)
        .await?;

    Ok(())
}
