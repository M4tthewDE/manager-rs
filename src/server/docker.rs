use anyhow::Result;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper_util::client::legacy::Client;
use hyperlocal::UnixClientExt;
use hyperlocal::{UnixConnector, Uri};
use prost::bytes::Buf;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Container {
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

#[derive(Deserialize, Debug)]
pub struct Port {
    #[serde(rename = "PrivatePort")]
    pub private_port: i64,

    #[serde(rename = "PublicPort")]
    pub public_port: i64,

    #[serde(rename = "Type")]
    pub port_type: String,
}

pub async fn list_containers() -> Result<Vec<Container>> {
    let url = Uri::new("/var/run/docker.sock", "/v1.47/containers/json?all=true").into();

    let client: Client<UnixConnector, Full<Bytes>> = Client::unix();

    let res = client.get(url).await?;
    let body = res.collect().await?.aggregate();

    let containers = serde_json::from_reader(body.reader())?;

    Ok(containers)
}
