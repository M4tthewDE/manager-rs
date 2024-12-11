use config::Config;
use tonic::transport::Server;
use tracing::info;

mod config;
mod docker;
mod system;

mod proto {
    tonic::include_proto!("manager");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let config = Config::new("config.toml".into())?;

    info!("Starting server at {:?}", config.address);

    Server::builder()
        .add_service(docker::service())
        .add_service(system::service())
        .serve(config.address)
        .await?;

    Ok(())
}
