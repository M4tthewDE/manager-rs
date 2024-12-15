use crate::state::info::Info;
use anyhow::Result;
use lib::proto::{system_client::SystemClient, Empty};

pub async fn get_info(server_address: String) -> Result<Info> {
    let mut client = SystemClient::connect(server_address).await?;
    let request = tonic::Request::new(Empty {});
    let response = client.get_info(request).await?;
    Ok(Info::from(response.get_ref()))
}
