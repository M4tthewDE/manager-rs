use memory_proto::{
    memory_server::{Memory, MemoryServer},
    Empty, MemoryReply,
};
use tonic::{Request, Response, Status};
use tracing::info;

use anyhow::{Context, Result};
use std::path::PathBuf;

use procfs::{FromRead, Meminfo};

mod memory_proto {
    tonic::include_proto!("memory");
}

#[derive(Debug, Default)]
pub struct MemoryService {}

#[tonic::async_trait]
impl Memory for MemoryService {
    async fn get_memory(&self, request: Request<Empty>) -> Result<Response<MemoryReply>, Status> {
        info!("REQUEST: {request:?}");
        let meminfo = Meminfo::from_file(PathBuf::from("/proc/meminfo"))
            .map_err(|e| Status::from_error(e.into()))?;

        Ok(Response::new(MemoryReply {
            total: meminfo.mem_total,
            free: meminfo.mem_free,
            available: meminfo
                .mem_available
                .context("no available memory found")
                .map_err(|e| Status::from_error(e.into()))?,
        }))
    }
}

pub fn service() -> MemoryServer<MemoryService> {
    MemoryServer::new(MemoryService::default())
}
