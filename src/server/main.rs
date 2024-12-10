use std::net::ToSocketAddrs;

use tonic::transport::Server;
use tracing::info;

mod docker;
mod system;

mod proto {
    tonic::include_proto!("manager");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let addr = "0.0.0.0:8080".to_socket_addrs()?.next().unwrap();

    info!("Starting server at {addr:?}");

    Server::builder()
        .add_service(docker::service())
        .add_service(system::service())
        .serve(addr)
        .await?;

    Ok(())
}
