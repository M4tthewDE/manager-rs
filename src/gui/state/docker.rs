use super::docker_proto;

pub struct Container {
    pub name: String,
    pub image: String,
    pub status: String,
}

impl Container {
    pub fn new(c: &docker_proto::Container) -> Self {
        Self {
            name: c.names.first().unwrap_or(&"".to_string()).to_string(),
            image: c.image.clone(),
            status: c.status.clone(),
        }
    }
}
