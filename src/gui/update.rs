use anyhow::Result;
use std::{
    sync::mpsc::Sender,
    time::{Duration, Instant},
};
use tracing::warn;

use crate::{client::info, config::Config, state::State};

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
