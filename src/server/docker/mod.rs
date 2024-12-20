use serde::Deserialize;

pub mod container;
pub mod image;
pub mod version;

const DOCKER_SOCK: &str = "/var/run/docker.sock";

#[derive(Deserialize, Debug)]
struct Error {
    #[allow(dead_code)]
    message: String,
}
