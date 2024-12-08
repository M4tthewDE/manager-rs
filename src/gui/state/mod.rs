use anyhow::Result;
use cpu::Cpu;
use docker::Container;
use futures::future::{self, BoxFuture};
use info::Info;
use memory::{Disk, Memory};
use proto::{docker_client::DockerClient, ContainerIdentifier, Empty};
use proto::{system_client::SystemClient, MemoryReply};
use std::sync::mpsc::Sender;

pub mod cpu;
pub mod docker;
pub mod info;
pub mod memory;

mod proto {
    tonic::include_proto!("manager");
}

pub type StateChangeMessage = Box<dyn FnOnce(&mut State) + Send + Sync>;

#[derive(Default)]
pub struct State {
    pub containers: Vec<Container>,
    pub memory: Memory,
    pub disks: Vec<Disk>,
    pub info: Info,
    pub cpus: Vec<Cpu>,
}

pub async fn update(tx: Sender<StateChangeMessage>) -> Result<()> {
    let futures: Vec<BoxFuture<Result<StateChangeMessage>>> = vec![
        Box::pin(update_memory()),
        Box::pin(update_containers()),
        Box::pin(update_disks()),
        Box::pin(update_info()),
        Box::pin(update_cpus()),
    ];

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
    let mut client = SystemClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(proto::Empty {});
    let response = client.get_memory(request).await?;
    let memory = Memory::new(response.get_ref());

    Ok(Box::new(move |state: &mut State| {
        state.memory = memory;
    }))
}

async fn update_disks() -> Result<StateChangeMessage> {
    let mut client = SystemClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(proto::Empty {});
    let response = client.get_disks(request).await?;
    let disks = response.get_ref().disks.iter().map(Disk::new).collect();

    Ok(Box::new(move |state: &mut State| {
        state.disks = disks;
    }))
}

async fn update_info() -> Result<StateChangeMessage> {
    let mut client = SystemClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(proto::Empty {});
    let response = client.get_info(request).await?;
    let info = Info::new(response.get_ref());

    Ok(Box::new(move |state: &mut State| {
        state.info = info;
    }))
}

async fn update_cpus() -> Result<StateChangeMessage> {
    let mut client = SystemClient::connect("http://[::1]:50051").await?;
    let request = tonic::Request::new(proto::Empty {});
    let response = client.get_cpus(request).await?;
    let cpus = response.get_ref().cpus.iter().map(Cpu::new).collect();

    Ok(Box::new(move |state: &mut State| {
        state.cpus = cpus;
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
