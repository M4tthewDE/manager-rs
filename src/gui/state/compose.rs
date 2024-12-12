use super::proto;

#[derive(Debug)]
pub enum DiffResult {
    New,
    Same,
    Modified,
}

impl From<i32> for DiffResult {
    fn from(res: i32) -> Self {
        match res {
            0 => Self::New,
            1 => Self::Same,
            2 => Self::Modified,
            _ => Self::Same,
        }
    }
}

#[derive(Debug)]
pub struct ComposeFileDiff {
    pub name: String,
    pub result: DiffResult,
}

impl From<&proto::ComposeFileDiff> for ComposeFileDiff {
    fn from(diff: &proto::ComposeFileDiff) -> Self {
        Self {
            name: diff.clone().name,
            result: diff.result.into(),
        }
    }
}
