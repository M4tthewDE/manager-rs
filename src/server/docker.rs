use anyhow::bail;
use anyhow::Result;
use docker_proto::docker_server;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper_util::client::legacy::Client;
use hyperlocal::UnixClientExt;
use hyperlocal::{UnixConnector, Uri};
use prost::bytes::Buf;
use serde::Deserialize;
use tonic::{Request, Response, Status};

mod docker_proto {
    tonic::include_proto!("docker");
}

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

async fn list_containers() -> Result<Vec<Container>> {
    let url = Uri::new("/var/run/docker.sock", "/v1.47/containers/json?all=true").into();

    let client: Client<UnixConnector, Full<Bytes>> = Client::unix();

    let res = client.get(url).await?;
    let body = res.collect().await?.aggregate();

    let containers = serde_json::from_reader(body.reader())?;

    Ok(containers)
}

#[derive(Deserialize, Debug)]
struct Error {
    #[allow(dead_code)]
    message: String,
}

async fn remove_container(id: &str) -> Result<()> {
    let url = Uri::new("/var/run/docker.sock", &format!("/v1.47/containers/{}", id));
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

#[derive(Debug, Default)]
pub struct DockerService {}

#[tonic::async_trait]
impl docker_server::Docker for DockerService {
    async fn list_containers(
        &self,
        _: Request<docker_proto::Empty>,
    ) -> Result<Response<docker_proto::ContainerListReply>, Status> {
        let containers = list_containers()
            .await
            .map_err(|e| Status::from_error(e.into()))?;

        let container_list: Vec<docker_proto::Container> = containers
            .iter()
            .map(|c| docker_proto::Container {
                id: c.id.clone(),
                names: c.names.clone(),
                image: c.image.clone(),
                command: c.command.clone(),
                created: c.created,
                ports: c
                    .ports
                    .iter()
                    .map(|p| docker_proto::Port {
                        private_port: p.private_port,
                        public_port: p.public_port,
                        port_type: p.port_type.clone(),
                    })
                    .collect(),
                status: c.status.clone(),
            })
            .collect();

        Ok(Response::new(docker_proto::ContainerListReply {
            container_list,
        }))
    }

    async fn remove_container(
        &self,
        request: Request<docker_proto::ContainerIdentifier>,
    ) -> Result<Response<docker_proto::Empty>, Status> {
        remove_container(&request.get_ref().id)
            .await
            .map_err(|e| Status::from_error(e.into()))?;

        Ok(Response::new(docker_proto::Empty {}))
    }
}

pub fn service() -> docker_server::DockerServer<DockerService> {
    docker_server::DockerServer::new(DockerService::default())
}
