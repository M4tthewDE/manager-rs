use std::fs::DirEntry;

use crate::config::Config;
use anyhow::{Context, Result};

use super::{
    proto::{self, compose_client::ComposeClient, ComposeFile, DiffRequest},
    State, StateChangeMessage,
};

#[derive(Debug)]
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

impl ComposeFile {
    fn new(dir_entry: DirEntry) -> Result<ComposeFile> {
        Ok(ComposeFile {
            name: dir_entry
                .file_name()
                .to_str()
                .context("invalid file name {p:?}")?
                .to_string(),
            content: std::fs::read_to_string(dir_entry.path())?,
        })
    }
}

pub async fn update_files(config: Config) -> Result<StateChangeMessage> {
    let mut files = Vec::new();
    for dir_entry in config.docker_compose_path.read_dir()? {
        files.push(ComposeFile::new(dir_entry?)?);
    }

    let diffs = diff_files(files, config.server_address).await?;

    Ok(Box::new(move |state: &mut State| {
        state.compose_file_diffs = diffs;
    }))
}

async fn diff_files(
    files: Vec<ComposeFile>,
    server_address: String,
) -> Result<Vec<ComposeFileDiff>> {
    let mut client = ComposeClient::connect(server_address).await?;

    let request = tonic::Request::new(DiffRequest { files });
    let res = client.diff(request).await?;

    Ok(res
        .get_ref()
        .diffs
        .iter()
        .map(ComposeFileDiff::from)
        .collect())
}
