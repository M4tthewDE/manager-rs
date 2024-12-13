use crate::state::proto::{self, docker_client::DockerClient, Empty};
use anyhow::Result;

pub struct Version {
    pub version: String,
    pub api_version: String,
}

impl From<&proto::VersionReply> for Version {
    fn from(v: &proto::VersionReply) -> Self {
        Version {
            version: v.version.clone(),
            api_version: v.api_version.clone(),
        }
    }
}

impl Default for Version {
    fn default() -> Self {
        Self {
            version: "n/a".to_string(),
            api_version: "n/a".to_string(),
        }
    }
}

pub async fn get_version(server_address: String) -> Result<Version> {
    let mut client = DockerClient::connect(server_address).await?;
    let request = tonic::Request::new(Empty {});
    Ok(Version::from(client.version(request).await?.get_ref()))
}
