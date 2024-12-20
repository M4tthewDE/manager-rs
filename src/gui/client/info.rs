use crate::proto::LogReply;
use crate::proto::{system_client::SystemClient, Empty};
use crate::state::info::Info;
use anyhow::Result;
use tonic::Streaming;

pub async fn get_info(server_address: String) -> Result<Info> {
    let mut client = SystemClient::connect(server_address).await?;
    let request = tonic::Request::new(Empty {});
    let response = client.get_info(request).await?;
    Ok(Info::from(response.get_ref()))
}

pub async fn stream_logs(server_address: String) -> Result<Streaming<LogReply>> {
    let mut client = SystemClient::connect(server_address).await?;
    let request = tonic::Request::new(Empty {});
    let stream = client.log(request).await?.into_inner();
    Ok(stream)
}
