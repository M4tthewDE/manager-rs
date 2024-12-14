use crate::client;
use anyhow::Result;
use compose::ComposeFileDiff;
use docker::DockerState;
use futures::future::{self, BoxFuture};
use info::Info;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use tracing::warn;

use crate::config::Config;

pub mod compose;
pub mod docker;
pub mod info;

pub mod proto {
    tonic::include_proto!("manager");
}

pub type StateChangeMessage = Box<dyn FnOnce(&mut State) + Send + Sync>;

#[derive(Default)]
pub struct State {
    pub docker_state: DockerState,
    pub info: Info,
    pub compose_file_diffs: Vec<ComposeFileDiff>,
}

pub async fn update(tx: Sender<StateChangeMessage>, config: Config) -> Result<()> {
    let start = Instant::now();
    let futures: Vec<BoxFuture<Result<StateChangeMessage>>> = vec![
        Box::pin(client::docker::update(config.server_address.clone())),
        Box::pin(client::info::update_info(config.server_address.clone())),
        Box::pin(client::compose::update_files(config)),
    ];

    for result in future::join_all(futures).await {
        tx.send(result?)?;
    }

    let elapsed = Instant::now() - start;
    if elapsed > Duration::from_millis(500) {
        warn!("Update time: {elapsed:?}");
    }

    Ok(())
}
