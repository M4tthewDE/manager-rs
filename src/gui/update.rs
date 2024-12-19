use anyhow::{Context, Result};
use std::{
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

    for dir_entry in config.docker_compose_path.read_dir()? {
        let dir_entry = dir_entry?;
        files.push(ComposeFile {
            name: dir_entry
                .file_name()
                .to_str()
                .context("invalid file name {p:?}")?
                .to_string(),
            content: std::fs::read_to_string(dir_entry.path())?,
        });
    }

    let diffs = crate::client::compose::diff_files(files, config.server_address).await?;

    Ok(tx.send(Box::new(move |state: &mut State| {
        state.compose_file_diffs = diffs;
    }))?)
}
