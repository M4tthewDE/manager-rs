use config::Config;
use std::sync::{Arc, Mutex};
use subscriber::{relay::LogRelay, StreamingLayer};
use tonic::transport::Server;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::{self},
    prelude::*,
    EnvFilter,
};

mod config;
mod docker;
mod service;
mod subscriber;

mod proto {
    tonic::include_proto!("manager");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_relay = setup_logging();
    let config = Config::new("config.toml".into())?;

    info!("Starting server at {:?}", config.address);

    Server::builder()
        .add_service(service::docker())
        .add_service(service::system(config.clone(), log_relay))
        .add_service(service::compose(config.clone()))
        .serve(config.address)
        .await?;

    Ok(())
}

fn setup_logging() -> Arc<Mutex<LogRelay>> {
    let fmt_layer = fmt::layer();
    let log_relay = Arc::new(Mutex::new(LogRelay::default()));
    let streaming_layer = StreamingLayer::new(Arc::clone(&log_relay));

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(streaming_layer)
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    log_relay
}
