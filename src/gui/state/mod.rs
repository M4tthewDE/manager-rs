use compose::ComposeFileDiff;
use docker::DockerState;
use info::Info;

pub mod compose;
pub mod docker;
pub mod info;

#[derive(Default)]
pub struct State {
    pub docker_state: DockerState,
    pub info: Info,
    pub compose_file_diffs: Vec<ComposeFileDiff>,
}
