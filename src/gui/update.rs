use anyhow::{Context, Result};
use futures::future::BoxFuture;
use std::{
    fs::DirEntry,
    sync::mpsc::Sender,
    time::{Duration, Instant},
};
use tracing::warn;

use crate::{
    client::{compose, info},
    config::Config,
    proto::ComposeFile,
    state::State,
};

pub type StateChangeMessage = Box<dyn FnOnce(&mut State) + Send + Sync>;

pub async fn update(tx: Sender<StateChangeMessage>, config: Config) -> Result<()> {
    let start = Instant::now();
    let futures: Vec<BoxFuture<Result<StateChangeMessage>>> = vec![
        Box::pin(update_info(config.server_address.clone())),
        Box::pin(update_compose(config)),
    ];

    for result in futures::future::join_all(futures).await {
        tx.send(result?)?;
    }

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

async fn update_compose(config: Config) -> Result<StateChangeMessage> {
    let mut files = Vec::new();
    for dir_entry in config.docker_compose_path.read_dir()? {
        files.push(ComposeFile::new(dir_entry?)?);
    }

    let diffs = compose::diff_files(files, config.server_address).await?;

    Ok(Box::new(move |state: &mut State| {
        state.compose_file_diffs = diffs;
    }))
}

impl ComposeFile {
    pub fn new(dir_entry: DirEntry) -> Result<Self> {
        Ok(Self {
            name: dir_entry
                .file_name()
                .to_str()
                .context("invalid file name {p:?}")?
                .to_string(),
            content: std::fs::read_to_string(dir_entry.path())?,
        })
    }
}
