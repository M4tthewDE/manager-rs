use crate::proto::{
    system_server::{System, SystemServer},
    Disk, DiskReply, Empty, InfoReply, MemoryReply,
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

    async fn get_info(&self, _: Request<Empty>) -> Result<Response<InfoReply>, Status> {
        Ok(Response::new(InfoReply {
            name: sysinfo::System::name().unwrap_or_default(),
            kernel_version: sysinfo::System::kernel_version().unwrap_or_default(),
            os_version: sysinfo::System::os_version().unwrap_or_default(),
            host_name: sysinfo::System::host_name().unwrap_or_default(),
        }))
    }
}

pub fn service() -> SystemServer<SystemService> {
    SystemServer::new(SystemService::default())
}
