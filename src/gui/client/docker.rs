use crate::state::docker::{container::Container, version::Version};

use crate::proto::{docker_client::DockerClient, ContainerIdentifier, Empty};

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

pub async fn get_containers(server_address: String) -> Result<Vec<Container>> {
    let mut client = DockerClient::connect(server_address.clone()).await?;
    let request = tonic::Request::new(Empty {});
    let response = client.list_containers(request).await?;

    let mut containers = Vec::new();
    for c in &response.get_ref().container_list {
        let logs = get_logs(c.id.clone(), server_address.clone()).await?;
        containers.push(Container::new(c, logs)?);
    }

    Ok(containers)
}

async fn get_logs(id: String, server_address: String) -> Result<Vec<String>> {
    let mut client = DockerClient::connect(server_address).await?;
    let request = tonic::Request::new(ContainerIdentifier { id });
    let response = client.logs_container(request).await?;
    Ok(response.get_ref().lines.clone())
}

pub async fn get_version(server_address: String) -> Result<Version> {
    let mut client = DockerClient::connect(server_address).await?;
    let request = tonic::Request::new(Empty {});
    Ok(Version::from(client.version(request).await?.get_ref()))
}
