use anyhow::{Context, Result};
use std::{
    path::{Path, PathBuf},
    sync::mpsc::Sender,
    time::{Duration, Instant},
};
use tracing::warn;

use crate::{client::info, config::Config, proto::ComposeFile, state::State};

pub type StateChangeMessage = Box<dyn FnOnce(&mut State) + Send + Sync>;

pub async fn update(tx: Sender<StateChangeMessage>, config: Config) -> Result<()> {
    let start = Instant::now();

    let state_change_message = update_info(config.server_address.clone()).await?;
    tx.send(state_change_message)?;

    let elapsed = Instant::now() - start;
    if elapsed > Duration::from_millis(500) {
        warn!("Update time: {elapsed:?}");
    }

    Ok(())
}

async fn update_info(server_address: String) -> Result<StateChangeMessage> {
    let info = info::get_info(server_address).await?;

    Ok(Box::new(move |state: &mut State| {
        state.info = info;
    }))
}

pub async fn update_compose_diffs(config: Config, tx: Sender<StateChangeMessage>) -> Result<()> {
    let mut files = Vec::new();

    gather_files(
        &config.docker_compose_path,
        &config.docker_compose_path,
        &mut files,
    )?;
    let diffs = crate::client::compose::diff_files(files, config.server_address).await?;

    Ok(tx.send(Box::new(move |state: &mut State| {
        state.compose_file_diffs = diffs;
    }))?)
}

fn gather_files(root_path: &PathBuf, path: &Path, files: &mut Vec<ComposeFile>) -> Result<()> {
    for dir_entry in path.read_dir()? {
        let dir_entry = dir_entry?;
        if dir_entry.path().is_dir() {
            gather_files(root_path, &dir_entry.path(), files)?;
            continue;
        }

        files.push(ComposeFile {
            path: dir_entry
                .path()
                .strip_prefix(root_path)?
                .to_path_buf()
                .to_str()
                .context("invalid  path {p:?}")?
                .to_string(),
            content: std::fs::read_to_string(dir_entry.path())?,
        });
    }

    Ok(())
}
