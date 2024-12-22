use std::collections::VecDeque;

use tokio::{runtime::Handle, sync::mpsc::Sender};
use tonic::Status;
use tracing::error;
use uuid::Uuid;

use crate::proto::LogReply;

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
    pub fn relay(&mut self, reply: Result<LogReply, Status>) {
        if self.cache.len() == MAX_CACHE_SIZE {
            self.cache.pop_front();
        }

        self.cache.push_back(reply.clone());
        let senders = self.senders.clone();

        Handle::current().spawn(async move {
            for sender in senders {
                if let Err(err) = sender.send(reply.clone()).await {
                    error!("Log relay send error '{err:?}' to sender {:?}", sender.id);
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
                if let Err(err) = sender.send(reply.clone()).await {
                    error!(
                        "Log relay cache send error '{err:?}' to sender {:?}",
                        sender.id
                    );
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
