use compose::ComposeFileDiff;
use info::Info;
use log::ServerLog;

pub mod compose;
pub mod info;
pub mod log;

#[derive(Default)]
pub struct State {
    pub info: Info,
    pub server_log: ServerLog,
    pub compose_file_diffs: Vec<ComposeFileDiff>,
}
