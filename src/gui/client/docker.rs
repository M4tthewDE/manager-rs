use lib::proto::{docker_client::DockerClient, ContainerIdentifier};

use anyhow::Result;

pub async fn start_container(id: String, server_address: String) -> Result<()> {
    let mut client = DockerClient::connect(server_address).await?;
    let request = tonic::Request::new(ContainerIdentifier { id });
    client.start_container(request).await?;

    Ok(())
}

pub async fn stop_container(id: String, server_address: String) -> Result<()> {
    let mut client = DockerClient::connect(server_address).await?;
    let request = tonic::Request::new(ContainerIdentifier { id });
    client.stop_container(request).await?;

    Ok(())
}

pub async fn remove_container(id: String, server_address: String) -> Result<()> {
    let mut client = DockerClient::connect(server_address).await?;
    let request = tonic::Request::new(ContainerIdentifier { id });
    client.remove_container(request).await?;

    Ok(())
}
