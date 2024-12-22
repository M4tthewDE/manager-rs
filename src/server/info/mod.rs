use crate::proto::{Cpu, CpuInfo, Disk, DiskInfo, DockerInfo, InfoReply, MemoryInfo, Version};
use sysinfo::{CpuRefreshKind, Disks, RefreshKind};
use tonic::Status;

use anyhow::Result;

use crate::docker;

pub async fn info() -> Result<InfoReply> {
    let mut sys = sysinfo::System::new_with_specifics(
        RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
    );

    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_all();

    Ok(InfoReply {
        name: sysinfo::System::name().unwrap_or_default(),
        kernel_version: sysinfo::System::kernel_version().unwrap_or_default(),
        os_version: sysinfo::System::os_version().unwrap_or_default(),
        host_name: sysinfo::System::host_name().unwrap_or_default(),

        memory_info: Some(memory_info(&mut sys)),
        disk_info: Some(disk_info()),
        cpu_info: Some(cpu_info(&mut sys)),
        docker_info: Some(docker_info().await?),
    })
}

fn memory_info(sys: &mut sysinfo::System) -> MemoryInfo {
    MemoryInfo {
        total: sys.total_memory(),
        free: sys.free_memory(),
        available: sys.available_memory(),
        used: sys.used_memory(),
    }
}

fn disk_info() -> DiskInfo {
    DiskInfo {
        disks: Disks::new_with_refreshed_list()
            .list()
            .iter()
            .map(|d| Disk {
                name: d.name().to_str().unwrap_or_default().to_string(),
                kind: d.kind().to_string(),
                file_system: d.file_system().to_str().unwrap_or_default().to_string(),
                total_space: d.total_space(),
                available_space: d.available_space(),
            })
            .collect(),
    }
}

fn cpu_info(sys: &mut sysinfo::System) -> CpuInfo {
    CpuInfo {
        cpus: sys
            .cpus()
            .iter()
            .map(|c| Cpu {
                name: c.name().to_string(),
                cpu_usage: c.cpu_usage(),
                frequency: c.frequency(),
            })
            .collect(),
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
        let container = crate::proto::Container {
            id: c.id.clone(),
            names: c.names.clone(),
            image: c.image.clone(),
            command: c.command.clone(),
            created: c.created,
            ports: c.ports.iter().map(crate::proto::Port::from).collect(),
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
