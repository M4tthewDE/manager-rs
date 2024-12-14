use anyhow::Result;

use crate::{
    config::Config,
    state::{
        compose::ComposeFileDiff,
        proto::{self, compose_client::ComposeClient, ComposeFile, DiffRequest, PushRequest},
        State, StateChangeMessage,
    },
};

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

pub async fn push_file(server_address: String, file_diff: ComposeFileDiff) -> Result<()> {
    let mut client = ComposeClient::connect(server_address).await?;
    let request = tonic::Request::new(PushRequest {
        file: Some(proto::ComposeFile {
            name: file_diff.name.clone(),
            content: file_diff.content,
        }),
        diff_result: proto::DiffResult::from(file_diff.result).into(),
    });

    client.push(request).await?;
    Ok(())
}
