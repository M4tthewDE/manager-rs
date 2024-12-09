use crate::state::proto;
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
            ports: c.ports.iter().map(Port::new).collect(),
            logs,
        })
    }
}

pub struct Port {
    pub private_port: String,
    pub public_port: String,
    pub port_type: String,
}

impl Port {
    pub fn new(p: &proto::Port) -> Self {
        Self {
            private_port: p.private_port.to_string(),
            public_port: p.public_port.to_string(),
            port_type: p.port_type.clone(),
        }
    }
}
