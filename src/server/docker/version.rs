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
pub struct Version {
    #[serde(rename = "Version")]
    version: String,

    #[serde(rename = "ApiVersion")]
    api_version: String,
}

impl From<Version> for proto::Version {
    fn from(v: Version) -> Self {
        Self {
            version: v.version,
            api_version: v.api_version,
        }
    }
}

pub async fn version() -> Result<Version> {
    let url = Uri::new(DOCKER_SOCK, "/v1.47/version").into();
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
