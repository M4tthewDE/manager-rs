use anyhow::Result;
use docker::{Container, DockerState, Version};
use futures::future::{self, BoxFuture};
use info::Info;
use proto::system_client::SystemClient;
use proto::{docker_client::DockerClient, ContainerIdentifier, Empty};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use tracing::warn;

pub mod docker;
pub mod info;

mod proto {
    tonic::include_proto!("manager");
}

pub type StateChangeMessage = Box<dyn FnOnce(&mut State) + Send + Sync>;

#[derive(Default)]
pub struct State {
    pub docker_state: DockerState,
    pub info: Info,
}

pub async fn update(tx: Sender<StateChangeMessage>, server_address: String) -> Result<()> {
    let start = Instant::now();
    let futures: Vec<BoxFuture<Result<StateChangeMessage>>> = vec![
        Box::pin(update_containers(server_address.clone())),
        Box::pin(update_info(server_address.clone())),
        Box::pin(update_version(server_address.clone())),
    ];

    for result in future::join_all(futures).await {
        tx.send(result?)?;
    }

    let elapsed = Instant::now() - start;
    if elapsed > Duration::from_millis(500) {
        warn!("Update time: {elapsed:?}");
    }

    Ok(())
}

async fn update_containers(server_address: String) -> Result<StateChangeMessage> {
    let mut client = DockerClient::connect(server_address.clone()).await?;
    let request = tonic::Request::new(Empty {});
    let response = client.list_containers(request).await?;

    let mut containers = Vec::new();
    for c in &response.get_ref().container_list {
        let logs = get_logs(c.id.clone(), server_address.clone()).await?;
        containers.push(Container::new(c, logs)?);
    }

    Ok(Box::new(move |state: &mut State| {
        state.docker_state.containers = containers;
    }))
}

async fn update_version(server_address: String) -> Result<StateChangeMessage> {
    let mut client = DockerClient::connect(server_address).await?;
    let request = tonic::Request::new(Empty {});
    let version = Version::from(client.version(request).await?.get_ref());

    Ok(Box::new(move |state: &mut State| {
        state.docker_state.version = version;
    }))
}

async fn get_logs(id: String, server_address: String) -> Result<Vec<String>> {
    let mut client = DockerClient::connect(server_address).await?;
    let request = tonic::Request::new(ContainerIdentifier { id });
    let response = client.logs_container(request).await?;
    Ok(response.get_ref().lines.clone())
}

async fn update_info(server_address: String) -> Result<StateChangeMessage> {
    let mut client = SystemClient::connect(server_address).await?;
    let request = tonic::Request::new(proto::Empty {});
    let response = client.get_info(request).await?;
    let info = Info::from(response.get_ref());

    Ok(Box::new(move |state: &mut State| {
        state.info = info;
    }))
}

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
