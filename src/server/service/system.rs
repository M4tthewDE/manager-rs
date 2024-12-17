use lib::proto::{
    system_server::System, Cpu, CpuInfo, Disk, DiskInfo, DockerInfo, Empty, InfoReply, MemoryInfo,
    Version,
};
use sysinfo::{CpuRefreshKind, Disks, RefreshKind};
use tonic::{Request, Response, Status};

use anyhow::Result;

use crate::docker;

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
            docker_info: Some(docker_info().await?),
        }))
    }
}

async fn docker_info() -> Result<DockerInfo, Status> {
    let version: Version = docker::version::version()
        .await
        .map_err(|e| Status::from_error(e.into()))?
        .into();

    let containers = docker::container::list()
        .await
        .map_err(|e| Status::from_error(e.into()))?;

    let mut container_list = Vec::new();
    for c in containers {
        let logs = docker::container::logs(&c.id)
            .await
            .map_err(|e| Status::from_error(e.into()))?;
        let container = lib::proto::Container {
            id: c.id.clone(),
            names: c.names.clone(),
            image: c.image.clone(),
            command: c.command.clone(),
            created: c.created,
            ports: c.ports.iter().map(lib::proto::Port::from).collect(),
            status: c.status.clone(),
            logs,
        };
        container_list.push(container);
    }

    Ok(DockerInfo {
        version: Some(version),
        container_list,
    })
}
