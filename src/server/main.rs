use tonic::transport::Server;

mod docker;
mod memory;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let addr = "[::1]:50051".parse()?;

    Server::builder()
        .add_service(docker::service())
        .add_service(memory::service())
        .serve(addr)
        .await?;

    Ok(())
}
