use config::Config;
use tonic::transport::Server;
use tracing::info;

mod config;
mod docker;
mod service;

mod proto {
    tonic::include_proto!("manager");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let config = Config::new("config.toml".into())?;

    info!("Starting server at {:?}", config.address);

    Server::builder()
        .add_service(service::docker())
        .add_service(service::system(config.clone()))
        .add_service(service::compose(config.clone()))
        .serve(config.address)
        .await?;

    Ok(())
}
