use docker::{
    docker_server::{Docker, DockerServer},
    ContainerListReply, Empty,
};
use tonic::{transport::Server, Request, Response, Status};

mod docker {
    tonic::include_proto!("docker");
}

#[derive(Debug, Default)]
pub struct DockerService {}

#[tonic::async_trait]
impl Docker for DockerService {
    async fn list_containers(
        &self,
        request: Request<Empty>,
    ) -> Result<Response<ContainerListReply>, Status> {
        println!("REQUEST: {request:?}");

        Ok(Response::new(ContainerListReply {
            container_list: vec![],
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let docker = DockerService::default();

    Server::builder()
        .add_service(DockerServer::new(docker))
        .serve(addr)
        .await?;

    Ok(())
}
