use anyhow::Result;
use serde::Deserialize;
use tonic::{Request, Response, Status};

use crate::proto;

mod container;

#[derive(Deserialize, Debug)]
struct Error {
    #[allow(dead_code)]
    message: String,
}

#[derive(Debug, Default)]
pub struct DockerService {}

#[tonic::async_trait]
impl proto::docker_server::Docker for DockerService {
    async fn list_containers(
        &self,
        _: Request<proto::Empty>,
    ) -> Result<Response<proto::ContainerListReply>, Status> {
        let containers = container::list()
            .await
            .map_err(|e| Status::from_error(e.into()))?;

        let container_list: Vec<proto::Container> =
            containers.iter().map(proto::Container::from).collect();

        Ok(Response::new(proto::ContainerListReply { container_list }))
    }

    async fn start_container(
        &self,
        request: Request<proto::ContainerIdentifier>,
    ) -> Result<Response<proto::Empty>, Status> {
        container::start(&request.get_ref().id)
            .await
            .map_err(|e| Status::from_error(e.into()))?;

        Ok(Response::new(proto::Empty {}))
    }

    async fn stop_container(
        &self,
        request: Request<proto::ContainerIdentifier>,
    ) -> Result<Response<proto::Empty>, Status> {
        container::stop(&request.get_ref().id)
            .await
            .map_err(|e| Status::from_error(e.into()))?;

        Ok(Response::new(proto::Empty {}))
    }

    async fn remove_container(
        &self,
        request: Request<proto::ContainerIdentifier>,
    ) -> Result<Response<proto::Empty>, Status> {
        container::remove(&request.get_ref().id)
            .await
            .map_err(|e| Status::from_error(e.into()))?;

        Ok(Response::new(proto::Empty {}))
    }

    async fn logs_container(
        &self,
        request: Request<proto::ContainerIdentifier>,
    ) -> Result<Response<proto::LogsReply>, Status> {
        let lines = container::logs(&request.get_ref().id)
            .await
            .map_err(|e| Status::from_error(e.into()))?;

        Ok(Response::new(proto::LogsReply { lines }))
    }
}

pub fn service() -> proto::docker_server::DockerServer<DockerService> {
    proto::docker_server::DockerServer::new(DockerService::default())
}
