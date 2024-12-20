use std::sync::{Arc, Mutex};

use tokio::{runtime::Handle, sync::mpsc::Sender};
use tonic::Status;
use tracing::{error, Subscriber};
use tracing_subscriber::Layer;

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
                        Ok(log_relay) => log_relay.relay(Ok(LogReply {
                            level: LogLevel::Info.into(),
                            text: format!("{:?}", value),
                        })),
                        Err(err) => {
                            error!("{err:?}");
                        }
                    }
                }
            },
        );
    }
}

#[derive(Default)]
pub struct LogRelay {
    senders: Vec<Sender<Result<LogReply, Status>>>,
}

impl LogRelay {
    fn relay(&self, reply: Result<LogReply, Status>) {
        let reply = reply.clone();
        let senders = self.senders.clone();

        Handle::current().spawn(async move {
            for sender in &senders {
                match sender.send(reply.clone()).await {
                    Ok(_) => {}
                    Err(err) => {
                        error!(
                            "log relay send error, we should probably remove the  sender: {err:?}"
                        )
                    }
                }
            }
        });
    }

    pub fn add_sender(&mut self, sender: Sender<Result<LogReply, Status>>) {
        self.senders.push(sender);
    }
}
