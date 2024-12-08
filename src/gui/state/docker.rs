use anyhow::Result;
use chrono::DateTime;
use chrono_humanize::HumanTime;

use super::docker_proto;

pub struct Container {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub created: String,
}

impl Container {
    pub fn new(c: &docker_proto::Container) -> Result<Self> {
        Ok(Self {
            id: c.id.clone(),
            name: c.names.join(", "),
            image: c.image.clone(),
            status: c.status.clone(),
            created: HumanTime::from(DateTime::from_timestamp(c.created, 0).unwrap()).to_string(),
        })
    }
}
