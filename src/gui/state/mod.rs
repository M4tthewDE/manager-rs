use anyhow::Result;
use docker::Container;
use docker_proto::{docker_client::DockerClient, ContainerListReply, Empty};
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

pub type StateChangeMessage = Box<dyn FnMut(&mut State) + Send + Sync>;

#[derive(Default)]
pub struct State {
    pub containers: Vec<Container>,
    pub memory: Memory,
}

pub async fn update(tx: Sender<StateChangeMessage>) -> Result<()> {
    let container_reply = get_containers().await?;
    tx.send(Box::new(move |state: &mut State| {
        state.containers = container_reply
            .container_list
            .iter()
            .map(Container::new)
            .collect()
    }))?;

    let memory_reply = get_memory().await?;
    tx.send(Box::new(move |state: &mut State| {
        state.memory = Memory::new(&memory_reply);
    }))?;

    Ok(())
}

async fn get_containers() -> Result<ContainerListReply> {
    let mut client = DockerClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(Empty {});
    let response = client.list_containers(request).await?;
    Ok(response.get_ref().clone())
}

async fn get_memory() -> Result<MemoryReply> {
    let mut client = MemoryClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(memory_proto::Empty {});
    let response = client.get_memory(request).await?;
    Ok(*response.get_ref())
}
