fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("proto/docker.proto")?;
    tonic_build::compile_protos("proto/memory.proto")?;
    Ok(())
}
