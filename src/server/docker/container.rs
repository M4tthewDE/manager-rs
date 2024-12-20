use serde::Serialize;
use std::collections::HashMap;
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

use crate::docker::Error;
use crate::proto;

use super::DOCKER_SOCK;

#[derive(Deserialize, Debug)]
pub struct Port {
    #[serde(rename = "PrivatePort")]
    private_port: i64,

    #[serde(rename = "PublicPort")]
    public_port: i64,

    #[serde(rename = "Type")]
    port_type: String,
}

impl From<&Port> for proto::Port {
    fn from(p: &Port) -> Self {
        Self {
            private_port: p.private_port,
            public_port: p.public_port,
            port_type: p.port_type.clone(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct Container {
    #[serde(rename = "Id")]
    pub id: String,

    #[serde(rename = "Names")]
    pub names: Vec<String>,

    #[serde(rename = "Image")]
    pub image: String,

    #[serde(rename = "Command")]
    pub command: String,

    #[serde(rename = "Created")]
    pub created: i64,

    #[serde(rename = "Ports")]
    pub ports: Vec<Port>,

    #[serde(rename = "Status")]
    pub status: String,
}

pub async fn list() -> Result<Vec<Container>> {
    let url = Uri::new(DOCKER_SOCK, "/v1.47/containers/json?all=true").into();
    let client: Client<UnixConnector, Full<Bytes>> = Client::unix();

    let res = client.get(url).await?;
    if res.status() != 200 {
        let body = res.collect().await?.aggregate();
        let error: Error = serde_json::from_reader(body.reader())?;
        bail!("{error:?}")
    }

    let body = res.collect().await?.aggregate();
    Ok(serde_json::from_reader(body.reader())?)
}

pub async fn start(id: &str) -> Result<()> {
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

pub async fn stop(id: &str) -> Result<()> {
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

pub async fn remove(id: &str) -> Result<()> {
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

pub async fn logs(id: &str) -> Result<Vec<String>> {
    let url = Uri::new(
        DOCKER_SOCK,
        &format!(
            "/v1.47/containers/{}/logs?stdout=true&timestamps=true&tail=1000",
            id
        ),
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

#[derive(Serialize, Debug)]
pub struct PortBinding {
    #[serde(rename = "HostIp")]
    pub host_ip: String,

    #[serde(rename = "HostPort")]
    pub host_port: String,
}

#[derive(Serialize, Debug)]
pub struct HostConfig {
    #[serde(rename = "PortBindings")]
    pub port_bindings: HashMap<String, Vec<PortBinding>>,

    #[serde(rename = "Binds")]
    pub binds: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct ContainerCreationBody {
    #[serde(rename = "Image")]
    pub image: String,

    #[serde(rename = "Cmd")]
    pub command: Option<String>,

    #[serde(rename = "HostConfig")]
    pub host_config: HostConfig,
}

#[derive(Deserialize, Debug)]
pub struct ContainerCreationResponse {
    #[serde(rename = "Id")]
    pub id: String,
}

pub async fn create(name: &str, body: ContainerCreationBody) -> Result<String> {
    let url = Uri::new(
        DOCKER_SOCK,
        &format!("/v1.47/containers/create?name={}", name),
    );

    let req = hyper::Request::builder()
        .uri(url)
        .method("POST")
        .header("Content-Type", "application/json")
        .body(Full::from(serde_json::to_string(&body)?))?;

    let client: Client<UnixConnector, Full<Bytes>> = Client::unix();
    let res = client.request(req).await?;
    if res.status() != 201 {
        let status = res.status();
        let body = res.collect().await?.aggregate();
        let error: Error = serde_json::from_reader(body.reader())?;
        bail!("status: {status}, {error:?}")
    }

    let body = res.collect().await?.aggregate();
    let container_creation_response: ContainerCreationResponse =
        serde_json::from_reader(body.reader())?;

    Ok(container_creation_response.id)
}
