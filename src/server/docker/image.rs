use std::io::Read;

use anyhow::bail;
use anyhow::Result;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper_util::client::legacy::Client;
use hyperlocal::UnixClientExt;
use hyperlocal::{UnixConnector, Uri};
use prost::bytes::Buf;

use crate::docker::Error;

use super::DOCKER_SOCK;

pub async fn pull(name: &str, tag: &str) -> Result<()> {
    let url = Uri::new(
        DOCKER_SOCK,
        &format!("/v1.47/images/create?fromImage={name}&tag={tag}"),
    );

    let req = hyper::Request::builder()
        .uri(url)
        .method("POST")
        .body(Full::from(""))?;

    let client: Client<UnixConnector, Full<Bytes>> = Client::unix();
    let res = client.request(req).await?;
    if res.status() != 200 {
        let status = res.status();
        let body = res.collect().await?.aggregate();
        let error: Error = serde_json::from_reader(body.reader())?;
        bail!("status: {status}, {error:?}")
    }

    let body = res.collect().await?.aggregate();
    let mut dest = String::new();
    body.reader().read_to_string(&mut dest)?;

    Ok(())
}
