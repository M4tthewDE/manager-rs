use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::proto::DeployRequest;
use crate::state::compose::ComposeFileDiff;

use crate::proto::{self, compose_client::ComposeClient, ComposeFile, DiffRequest, PushRequest};

pub async fn diff_files(
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

pub async fn push_file(server_address: String, file_diff: ComposeFileDiff) -> Result<()> {
    let mut client = ComposeClient::connect(server_address).await?;
    let request = tonic::Request::new(PushRequest {
        file: Some(proto::ComposeFile {
            path: file_diff
                .path
                .clone()
                .to_str()
                .context("invalid path {file_diff:?}")?
                .to_string(),
            content: file_diff.content,
        }),
        diff_result: proto::DiffResult::from(file_diff.result).into(),
    });

    client.push(request).await?;
    Ok(())
}

pub async fn deploy(server_address: String, path: PathBuf) -> Result<()> {
    let mut client = ComposeClient::connect(server_address).await?;
    let request = tonic::Request::new(DeployRequest {
        path: path.to_str().context("invalid path {path:?}")?.to_string(),
    });

    client.deploy(request).await?;
    Ok(())
}
