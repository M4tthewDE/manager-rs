use config::Config;
use tonic::transport::Server;
use tracing::info;

mod compose;
mod config;
mod docker;
mod system;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let config = Config::new("config.toml".into())?;

    info!("Starting server at {:?}", config.address);

    Server::builder()
        .add_service(docker::service())
        .add_service(system::service())
        .add_service(compose::service(config.clone()))
        .serve(config.address)
        .await?;

    Ok(())
}
