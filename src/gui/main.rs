use docker::{docker_client::DockerClient, Empty};

pub mod docker {
    tonic::include_proto!("docker");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DockerClient::connect("http://[::1]:50051").await?;

    let request = tonic::Request::new(Empty {});

    let response = client.list_containers(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
