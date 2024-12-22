use std::sync::{Arc, Mutex};

use relay::LogRelay;
use tracing::Subscriber;
use tracing_subscriber::Layer;

use crate::proto::{LogLevel, LogReply};

pub mod relay;

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
                        Ok(mut log_relay) => log_relay.relay(LogReply {
                            level: convert_level(event.metadata().level()).into(),
                            text: format!("{:?}", value),
                        }),
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
