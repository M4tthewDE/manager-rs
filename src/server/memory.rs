use memory_proto::{
    memory_server::{Memory, MemoryServer},
    Empty, MemoryReply,
};
use sysinfo::System;
use tonic::{Request, Response, Status};

use anyhow::Result;

mod memory_proto {
    tonic::include_proto!("memory");
}

#[derive(Debug, Default)]
pub struct MemoryService {}

#[tonic::async_trait]
impl Memory for MemoryService {
    async fn get_memory(&self, _: Request<Empty>) -> Result<Response<MemoryReply>, Status> {
        let mut sys = System::new_all();
        sys.refresh_all();

        Ok(Response::new(MemoryReply {
            total: sys.total_memory(),
            free: sys.free_memory(),
            available: sys.available_memory(),
            used: sys.used_memory(),
        }))
    }
}

pub fn service() -> MemoryServer<MemoryService> {
    MemoryServer::new(MemoryService::default())
}
