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
struct Port {
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

impl From<&Container> for proto::Container {
    fn from(c: &Container) -> Self {
        Self {
            id: c.id.clone(),
            names: c.names.clone(),
            image: c.image.clone(),
            command: c.command.clone(),
            created: c.created,
            ports: c.ports.iter().map(proto::Port::from).collect(),
            status: c.status.clone(),
        }
    }
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
