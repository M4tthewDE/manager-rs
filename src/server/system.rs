use crate::proto::{
    system_server::{System, SystemServer},
    Disk, DiskReply, Empty, MemoryReply,
};
use sysinfo::Disks;
use tonic::{Request, Response, Status};

use anyhow::Result;

#[derive(Debug, Default)]
pub struct SystemService {}

#[tonic::async_trait]
impl System for SystemService {
    async fn get_memory(&self, _: Request<Empty>) -> Result<Response<MemoryReply>, Status> {
        let mut sys = sysinfo::System::new_all();
        sys.refresh_all();

        Ok(Response::new(MemoryReply {
            total: sys.total_memory(),
            free: sys.free_memory(),
            available: sys.available_memory(),
            used: sys.used_memory(),
        }))
    }

    async fn get_disks(&self, _: Request<Empty>) -> Result<Response<DiskReply>, Status> {
        let mut sys = sysinfo::System::new_all();
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

pub fn service() -> SystemServer<SystemService> {
    SystemServer::new(SystemService::default())
}
