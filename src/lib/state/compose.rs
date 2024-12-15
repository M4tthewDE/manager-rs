use crate::proto;

#[derive(Debug, Clone)]
pub enum DiffResult {
    New,
    Same,
    Modified,
    Removed,
}

impl From<i32> for DiffResult {
    fn from(res: i32) -> Self {
        match res {
            0 => Self::New,
            1 => Self::Same,
            2 => Self::Modified,
            3 => Self::Removed,
            _ => Self::Same,
        }
    }
}

impl From<DiffResult> for proto::DiffResult {
    fn from(res: DiffResult) -> Self {
        match res {
            DiffResult::New => Self::New,
            DiffResult::Same => Self::Same,
            DiffResult::Modified => Self::Modified,
            DiffResult::Removed => Self::Removed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComposeFileDiff {
    pub name: String,
    pub result: DiffResult,
    pub content: String,
}

impl From<&proto::ComposeFileDiff> for ComposeFileDiff {
    fn from(diff: &proto::ComposeFileDiff) -> Self {
        Self {
            name: diff.clone().name,
            result: diff.result.into(),
            content: diff.clone().content,
        }
    }
}
