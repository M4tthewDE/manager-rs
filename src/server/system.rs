use lib::proto::{
    system_server::{System, SystemServer},
    Cpu, CpuInfo, Disk, DiskInfo, Empty, InfoReply, MemoryInfo,
};
use sysinfo::{CpuRefreshKind, Disks, RefreshKind};
use tonic::{Request, Response, Status};

use anyhow::Result;

#[derive(Debug, Default)]
pub struct SystemService {}

#[tonic::async_trait]
impl System for SystemService {
    async fn get_info(&self, _: Request<Empty>) -> Result<Response<InfoReply>, Status> {
        let mut sys = sysinfo::System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
        );

        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        sys.refresh_all();

        let cpus = sys.cpus().iter().map(Cpu::from).collect();
        let disks = Disks::new_with_refreshed_list()
            .list()
            .iter()
            .map(Disk::from)
            .collect();

        Ok(Response::new(InfoReply {
            name: sysinfo::System::name().unwrap_or_default(),
            kernel_version: sysinfo::System::kernel_version().unwrap_or_default(),
            os_version: sysinfo::System::os_version().unwrap_or_default(),
            host_name: sysinfo::System::host_name().unwrap_or_default(),

            memory_info: Some(MemoryInfo {
                total: sys.total_memory(),
                free: sys.free_memory(),
                available: sys.available_memory(),
                used: sys.used_memory(),
            }),
            disk_info: Some(DiskInfo { disks }),
            cpu_info: Some(CpuInfo { cpus }),
        }))
    }
}

pub fn service() -> SystemServer<SystemService> {
    SystemServer::new(SystemService::default())
}
