use std::io::BufRead;
use tracing::debug;

use anyhow::bail;
use anyhow::Result;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper_util::client::legacy::Client;
use hyperlocal::UnixClientExt;
use hyperlocal::{UnixConnector, Uri};
use prost::bytes::Buf;
use serde::Deserialize;
use tonic::{Request, Response, Status};

use crate::proto;

#[derive(Deserialize, Debug)]
struct Container {
    #[serde(rename = "Id")]
    id: String,

    #[serde(rename = "Names")]
    names: Vec<String>,

    #[serde(rename = "Image")]
    image: String,

    #[serde(rename = "Command")]
    command: String,

    #[serde(rename = "Created")]
    created: i64,

    #[serde(rename = "Ports")]
    ports: Vec<Port>,

    #[serde(rename = "Status")]
    status: String,
}

#[derive(Deserialize, Debug)]
struct Port {
    #[serde(rename = "PrivatePort")]
    private_port: i64,

    #[serde(rename = "PublicPort")]
    public_port: i64,

    #[serde(rename = "Type")]
    port_type: String,
}

const DOCKER_SOCK: &str = "/var/run/docker.sock";

#[derive(Deserialize, Debug)]
struct Error {
    #[allow(dead_code)]
    message: String,
}

async fn list_containers() -> Result<Vec<Container>> {
    let url = Uri::new(DOCKER_SOCK, "/v1.47/containers/json?all=true").into();
    let client: Client<UnixConnector, Full<Bytes>> = Client::unix();

    let res = client.get(url).await?;
    if res.status() != 200 {
        let body = res.collect().await?.aggregate();
        let error: Error = serde_json::from_reader(body.reader())?;
        bail!("{error:?}")
    }

    let body = res.collect().await?.aggregate();

    let containers = serde_json::from_reader(body.reader())?;

    Ok(containers)
}

async fn start_container(id: &str) -> Result<()> {
    let url = Uri::new(DOCKER_SOCK, &format!("/v1.47/containers/{}/start", id));
    let req = hyper::Request::builder()
        .uri(url)
        .method("POST")
        .body(Full::from(""))?;

    let client: Client<UnixConnector, Full<Bytes>> = Client::unix();
    let res = client.request(req).await?;
    if res.status() != 204 && res.status() != 304 {
        let body = res.collect().await?.aggregate();
        let error: Error = serde_json::from_reader(body.reader())?;
        bail!("{error:?}")
    }

    Ok(())
}

async fn stop_container(id: &str) -> Result<()> {
    let url = Uri::new(DOCKER_SOCK, &format!("/v1.47/containers/{}/stop", id));
    let req = hyper::Request::builder()
        .uri(url)
        .method("POST")
        .body(Full::from(""))?;

    let client: Client<UnixConnector, Full<Bytes>> = Client::unix();
    let res = client.request(req).await?;
    if res.status() != 204 && res.status() != 304 {
        let body = res.collect().await?.aggregate();
        let error: Error = serde_json::from_reader(body.reader())?;
        bail!("{error:?}")
    }

    Ok(())
}

async fn remove_container(id: &str) -> Result<()> {
    let url = Uri::new(DOCKER_SOCK, &format!("/v1.47/containers/{}", id));
    let req = hyper::Request::builder()
        .uri(url)
        .method("DELETE")
        .body(Full::from(""))?;

    let client: Client<UnixConnector, Full<Bytes>> = Client::unix();
    let res = client.request(req).await?;
    if res.status() != 204 {
        let body = res.collect().await?.aggregate();
        let error: Error = serde_json::from_reader(body.reader())?;
        bail!("{error:?}")
    }

    Ok(())
}

async fn container_logs(id: &str) -> Result<Vec<String>> {
    let url = Uri::new(
        DOCKER_SOCK,
        &format!("/v1.47/containers/{}/logs?stdout=true&timestamps=true", id),
    );

    let client: Client<UnixConnector, Full<Bytes>> = Client::unix();
    let res = client.get(url.into()).await?;

    if res.status() != 200 {
        let body = res.collect().await?.aggregate();
        let error: Error = serde_json::from_reader(body.reader())?;
        bail!("{error:?}")
    }

    let body = res.collect().await?.aggregate();
    let reader = body.reader();

    let mut lines = vec![];
    for l in reader.lines() {
        match l {
            Ok(line) => lines.push(line),
            Err(err) => debug!("Skipping log line: {err:?}"),
        }
    }

    Ok(lines)
}

impl From<&Container> for proto::Container {
    fn from(c: &Container) -> Self {
        Self {
            id: c.id.clone(),
            names: c.names.clone(),
            image: c.image.clone(),
            command: c.command.clone(),
            created: c.created,
            ports: c
                .ports
                .iter()
                .map(|p| proto::Port {
                    private_port: p.private_port,
                    public_port: p.public_port,
                    port_type: p.port_type.clone(),
                })
                .collect(),
            status: c.status.clone(),
        }
    }
}

#[derive(Debug, Default)]
pub struct DockerService {}

#[tonic::async_trait]
impl proto::docker_server::Docker for DockerService {
    async fn list_containers(
        &self,
        _: Request<proto::Empty>,
    ) -> Result<Response<proto::ContainerListReply>, Status> {
        let containers = list_containers()
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
        start_container(&request.get_ref().id)
            .await
            .map_err(|e| Status::from_error(e.into()))?;

        Ok(Response::new(proto::Empty {}))
    }

    async fn stop_container(
        &self,
        request: Request<proto::ContainerIdentifier>,
    ) -> Result<Response<proto::Empty>, Status> {
        stop_container(&request.get_ref().id)
            .await
            .map_err(|e| Status::from_error(e.into()))?;

        Ok(Response::new(proto::Empty {}))
    }

    async fn remove_container(
        &self,
        request: Request<proto::ContainerIdentifier>,
    ) -> Result<Response<proto::Empty>, Status> {
        remove_container(&request.get_ref().id)
            .await
            .map_err(|e| Status::from_error(e.into()))?;

        Ok(Response::new(proto::Empty {}))
    }

    async fn logs_container(
        &self,
        request: Request<proto::ContainerIdentifier>,
    ) -> Result<Response<proto::LogsReply>, Status> {
        let lines = container_logs(&request.get_ref().id)
            .await
            .map_err(|e| Status::from_error(e.into()))?;

        Ok(Response::new(proto::LogsReply { lines }))
    }
}

pub fn service() -> proto::docker_server::DockerServer<DockerService> {
    proto::docker_server::DockerServer::new(DockerService::default())
}
