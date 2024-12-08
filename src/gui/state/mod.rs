use anyhow::Result;
use docker::Container;
use docker_proto::{docker_client::DockerClient, ContainerIdentifier, Empty};
use futures::future::{self, BoxFuture};
use memory::Memory;
use memory_proto::{memory_client::MemoryClient, MemoryReply};
use std::sync::mpsc::Sender;

pub mod docker;
pub mod memory;

pub mod docker_proto {
    tonic::include_proto!("docker");
}

pub mod memory_proto {
    tonic::include_proto!("memory");
}

pub type StateChangeMessage = Box<dyn FnOnce(&mut State) + Send + Sync>;

#[derive(Default)]
pub struct State {
    pub containers: Vec<Container>,
    pub memory: Memory,
}

pub async fn update(tx: Sender<StateChangeMessage>) -> Result<()> {
    let futures: Vec<BoxFuture<Result<StateChangeMessage>>> =
        vec![Box::pin(update_memory()), Box::pin(update_containers())];

    for result in future::join_all(futures).await {
        tx.send(result?)?;
    }

    Ok(())
}

async fn update_containers() -> Result<StateChangeMessage> {
    let mut client = DockerClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(Empty {});
    let response = client.list_containers(request).await?;

    let mut containers = Vec::new();
    for c in &response.get_ref().container_list {
        let logs = get_logs(c.id.clone()).await?;
        containers.push(Container::new(c, logs)?);
    }

    Ok(Box::new(move |state: &mut State| {
        state.containers = containers;
    }))
}

async fn get_logs(id: String) -> Result<Vec<String>> {
    let mut client = DockerClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(ContainerIdentifier { id });
    let response = client.logs_container(request).await?;
    Ok(response.get_ref().lines.clone())
}

async fn update_memory() -> Result<StateChangeMessage> {
    let mut client = MemoryClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(memory_proto::Empty {});
    let response = client.get_memory(request).await?;
    let memory = Memory::new(response.get_ref());

    Ok(Box::new(move |state: &mut State| {
        state.memory = memory;
    }))
}

pub async fn start_container(id: String) -> Result<()> {
    let mut client = DockerClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(ContainerIdentifier { id });
    client.start_container(request).await?;

    Ok(())
}

pub async fn stop_container(id: String) -> Result<()> {
    let mut client = DockerClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(ContainerIdentifier { id });
    client.stop_container(request).await?;

    Ok(())
}

pub async fn remove_container(id: String) -> Result<()> {
    let mut client = DockerClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(ContainerIdentifier { id });
    client.remove_container(request).await?;

    Ok(())
}
