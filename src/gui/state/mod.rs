use compose::ComposeFileDiff;
use info::Info;

pub mod compose;
pub mod info;

#[derive(Default)]
pub struct State {
    pub info: Info,
    pub compose_file_diffs: Vec<ComposeFileDiff>,
}
