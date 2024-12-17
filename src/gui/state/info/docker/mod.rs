use container::Container;
use version::Version;

pub mod container;
pub mod version;

#[derive(Default)]
pub struct DockerInfo {
    pub containers: Vec<Container>,
    pub version: Version,
}

impl From<crate::proto::DockerInfo> for DockerInfo {
    fn from(docker_info: crate::proto::DockerInfo) -> Self {
        Self {
            containers: docker_info
                .clone()
                .container_list
                .iter()
                .map(|c| Container::new(c, c.logs.clone()))
                .collect(),
            version: Version::from(&docker_info.version.clone().unwrap_or_default()),
        }
    }
}
