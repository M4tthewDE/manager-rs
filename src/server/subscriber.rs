use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use tokio::{runtime::Handle, sync::mpsc::Sender};
use tonic::Status;
use tracing::{error, Subscriber};
use tracing_subscriber::Layer;
use uuid::Uuid;

use crate::proto::{LogLevel, LogReply};

pub struct StreamingLayer {
    relay: Arc<Mutex<LogRelay>>,
}

impl StreamingLayer {
    pub fn new(relay: Arc<Mutex<LogRelay>>) -> Self {
        Self { relay }
    }
}

impl<S: Subscriber> Layer<S> for StreamingLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        event.record(
            &mut |field: &tracing::field::Field, value: &dyn std::fmt::Debug| {
                if field.name() == "message" {
                    match self.relay.lock() {
                        Ok(mut log_relay) => log_relay.relay(Ok(LogReply {
                            level: convert_level(event.metadata().level()).into(),
                            text: format!("{:?}", value),
                        })),
                        Err(err) => {
                            eprintln!("relay lock error: {err:?}");
                        }
                    }
                }
            },
        );
    }
}

fn convert_level(level: &tracing::Level) -> LogLevel {
    match *level {
        tracing::Level::TRACE => LogLevel::Info,
        tracing::Level::DEBUG => LogLevel::Debug,
        tracing::Level::INFO => LogLevel::Info,
        tracing::Level::WARN => LogLevel::Warn,
        tracing::Level::ERROR => LogLevel::Error,
    }
}

#[derive(Clone)]
pub struct LogSender {
    id: Uuid,
    sender: Sender<Result<LogReply, Status>>,
}

impl LogSender {
    pub fn new(id: Uuid, sender: Sender<Result<LogReply, Status>>) -> Self {
        Self { id, sender }
    }

    async fn send(&self, reply: Result<LogReply, Status>) -> anyhow::Result<()> {
        Ok(self.sender.send(reply).await?)
    }
}

const MAX_CACHE_SIZE: usize = 1_000;

#[derive(Default)]
pub struct LogRelay {
    cache: VecDeque<Result<LogReply, Status>>,
    senders: Vec<LogSender>,
}

impl LogRelay {
    fn relay(&mut self, reply: Result<LogReply, Status>) {
        if self.cache.len() == MAX_CACHE_SIZE {
            self.cache.pop_front();
        }

        self.cache.push_back(reply.clone());

        let reply = reply.clone();
        let senders = self.senders.clone();

        Handle::current().spawn(async move {
            for sender in senders {
                match sender.send(reply.clone()).await {
                    Ok(_) => {}
                    Err(err) => {
                        error!("Log relay send error '{err:?}' to sender {:?}", sender.id);
                    }
                }
            }
        });
    }

    pub fn add_sender(&mut self, sender: LogSender) {
        self.send_cache(sender.clone());
        self.senders.push(sender);
    }

    fn send_cache(&self, sender: LogSender) {
        let cache = self.cache.clone();
        Handle::current().spawn(async move {
            for reply in &cache {
                match sender.send(reply.clone()).await {
                    Ok(_) => {}
                    Err(err) => {
                        error!(
                            "Log relay cache send error '{err:?}' to sender {:?}",
                            sender.id
                        );
                    }
                }
            }
        });
    }

    pub fn remove_sender(&mut self, id: Uuid) {
        match self.find_sender(id) {
            Some(i) => {
                self.senders.remove(i);
            }
            None => error!("Tried to remove sender which does not exist: {id}"),
        }
    }

    fn find_sender(&self, id: Uuid) -> Option<usize> {
        for (i, sender) in self.senders.iter().enumerate() {
            if sender.id == id {
                return Some(i);
            }
        }

        None
    }
}
