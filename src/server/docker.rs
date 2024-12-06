use http_body_util::BodyExt;
use prost::bytes::Buf;
use serde::Deserialize;
use std::error::Error;

use http_body_util::Full;
use hyper::body::Bytes;
use hyper_util::client::legacy::Client;
use hyperlocal::UnixClientExt;
use hyperlocal::{UnixConnector, Uri};

#[derive(Deserialize, Debug)]
pub struct Container {
    #[serde(rename = "Names")]
    pub names: Vec<String>,
    #[serde(rename = "Image")]
    pub image: String,
}

pub async fn list_containers() -> Result<Vec<Container>, Box<dyn Error + Send + Sync>> {
    let url = Uri::new("/var/run/docker.sock", "/v1.47/containers/json?all=true").into();

    let client: Client<UnixConnector, Full<Bytes>> = Client::unix();

    let res = client.get(url).await?;
    let body = res.collect().await?.aggregate();

    let containers = serde_json::from_reader(body.reader())?;

    Ok(containers)
}
