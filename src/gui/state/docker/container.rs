use crate::state::proto::{self, docker_client::DockerClient, ContainerIdentifier, Empty};
use anyhow::Result;
use chrono::DateTime;
use chrono_humanize::HumanTime;

pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub created: String,
    pub ports: Vec<Port>,
    pub logs: Vec<String>,
}

impl Container {
    pub fn new(c: &proto::Container, logs: Vec<String>) -> Result<Self> {
        let created = DateTime::from_timestamp(c.created, 0).unwrap();
        Ok(Self {
            id: c.id.clone(),
            name: c.names.join(", "),
            image: c.image.clone(),
            status: c.status.clone(),
            created: format!("{} ({:?})", HumanTime::from(created), created),
            ports: c.ports.iter().map(Port::from).collect(),
            logs,
        })
    }
}

pub struct Port {
    pub private_port: String,
    pub public_port: String,
    pub port_type: String,
}

impl From<&proto::Port> for Port {
    fn from(p: &proto::Port) -> Self {
        Self {
            private_port: p.private_port.to_string(),
            public_port: p.public_port.to_string(),
            port_type: p.port_type.clone(),
        }
    }
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
