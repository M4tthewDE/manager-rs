use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::mpsc::Sender;
use tokio_stream::{wrappers::ReceiverStream, Stream};
use uuid::Uuid;

use crate::{
    proto::{system_server::System, Empty, InfoReply, LogReply},
    subscriber::relay::{LogRelay, LogSender},
};
use tonic::{Request, Response, Status};

use anyhow::Result;
use tracing::{debug, error, info};

use crate::config::Config;

pub struct SystemService {
    info_reply: Arc<Mutex<InfoReply>>,
    log_relay: Arc<Mutex<LogRelay>>,
}

impl SystemService {
    pub fn new(config: Config, log_relay: Arc<Mutex<LogRelay>>) -> Self {
        let info_reply = Arc::new(Mutex::new(InfoReply::default()));
        let update_interval = Duration::from_millis(config.update_interval);

        let i = Arc::clone(&info_reply);
        tokio::task::spawn(async move {
            run_updater(update_interval, i).await;
        });

        Self {
            info_reply,
            log_relay,
        }
    }
}

async fn run_updater(update_interval: Duration, info: Arc<Mutex<InfoReply>>) {
    info!("Starting info updater with interval {:?}", update_interval);
    loop {
        tokio::time::sleep(update_interval).await;
        match crate::info::info().await {
            Ok(i) => match info.lock() {
                Ok(mut info) => *info = i,
                Err(err) => error!("{err:?}"),
            },
            Err(err) => error!("update error {err:?}"),
        }
    }
}

async fn run_close_watcher(
    tx: Sender<Result<LogReply, Status>>,
    relay: Arc<Mutex<LogRelay>>,
    id: Uuid,
) {
    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;

        debug!("Checking if channel {id} is closed");
        if tx.is_closed() {
            match relay.lock() {
                Ok(mut relay) => {
                    relay.remove_sender(id);
                    break;
                }
                Err(err) => {
                    error!("relay lock error: {err:?}");
                }
            }
        }
    }

    info!("Removed closed sender {id}");
}

#[tonic::async_trait]
impl System for SystemService {
    async fn get_info(&self, _: Request<Empty>) -> Result<Response<InfoReply>, Status> {
        return match self.info_reply.lock() {
            Ok(info_reply) => Ok(Response::new(info_reply.clone())),
            Err(err) => {
                error!("{err:?}");
                Err(Status::from_error("Lock is poisoned".into()))
            }
        };
    }

    type LogStream = Pin<Box<dyn Stream<Item = Result<LogReply, Status>> + Send>>;

    async fn log(
        &self,
        _: tonic::Request<Empty>,
    ) -> std::result::Result<tonic::Response<Self::LogStream>, tonic::Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(128);
        let id = Uuid::new_v4();
        let log_sender = LogSender::new(id, tx.clone());

        info!("Adding sender {id}");
        match self.log_relay.lock() {
            Ok(mut log_relay) => {
                log_relay.add_sender(log_sender);
            }
            Err(err) => {
                error!("relay lock error: {err:?}");
                return Err(Status::from_error("Lock is poisoned".into()));
            }
        }

        let relay = Arc::clone(&self.log_relay);
        tokio::spawn(async move {
            run_close_watcher(tx, relay, id).await;
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(output_stream) as Self::LogStream))
    }
}
