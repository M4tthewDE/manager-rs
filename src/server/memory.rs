use crate::proto::{
    memory_server::{Memory, MemoryServer},
    Disk, DiskReply, Empty, MemoryReply,
};
use sysinfo::{Disks, System};
use tonic::{Request, Response, Status};

use anyhow::Result;

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

    async fn get_disks(&self, _: Request<Empty>) -> Result<Response<DiskReply>, Status> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let disks = Disks::new_with_refreshed_list()
            .list()
            .iter()
            .map(|d| Disk {
                name: d.name().to_str().unwrap_or_default().to_string(),
                kind: d.kind().to_string(),
                file_system: d.file_system().to_str().unwrap_or_default().to_string(),
                total_space: d.total_space(),
                available_space: d.available_space(),
            })
            .collect();

        Ok(Response::new(DiskReply { disks }))
    }
}

pub fn service() -> MemoryServer<MemoryService> {
    MemoryServer::new(MemoryService::default())
}
