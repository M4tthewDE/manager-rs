use anyhow::Result;
use docker::Container;
use docker_proto::{docker_client::DockerClient, Empty};
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

pub type StateChangeMessage = Box<dyn Fn(&mut State) + Send + Sync>;

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

    Ok(Box::new(move |state: &mut State| {
        state.containers = response
            .get_ref()
            .container_list
            .iter()
            .map(Container::new)
            .collect()
    }))
}

async fn update_memory() -> Result<StateChangeMessage> {
    let mut client = MemoryClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(memory_proto::Empty {});
    let response = client.get_memory(request).await?;

    Ok(Box::new(move |state: &mut State| {
        state.memory = Memory::new(response.get_ref());
    }))
}
