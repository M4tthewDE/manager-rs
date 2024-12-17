use container::Container;
use version::Version;

pub mod container;
pub mod version;

#[derive(Default)]
pub struct DockerState {
    pub containers: Vec<Container>,
    pub version: Version,
}
