use anyhow::Result;
use tonic::{Request, Response, Status};

use lib::proto;

use crate::docker::container;

#[derive(Debug, Default)]
pub struct DockerService {}

#[tonic::async_trait]
impl proto::docker_server::Docker for DockerService {
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
