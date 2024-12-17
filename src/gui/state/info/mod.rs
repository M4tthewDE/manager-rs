use cpu::Cpu;
use disk::Disk;
use docker::DockerInfo;
use memory::Memory;

use crate::proto::InfoReply;

pub mod cpu;
pub mod disk;
pub mod docker;
pub mod memory;

#[derive(Default)]
pub struct Info {
    pub name: String,
    pub kernel_version: String,
    pub os_version: String,
    pub host_name: String,

    pub memory: Memory,
    pub disks: Vec<Disk>,
    pub cpus: Vec<Cpu>,
    pub docker_info: DockerInfo,
}

impl From<&InfoReply> for Info {
    fn from(i: &InfoReply) -> Self {
        Self {
            name: i.name.clone(),
            kernel_version: i.kernel_version.clone(),
            os_version: i.os_version.clone(),
            host_name: i.host_name.clone(),

            memory: i.memory_info.unwrap_or_default().into(),
            disks: disks(i.clone()),
            cpus: cpus(i.clone()),
            docker_info: i.docker_info.clone().unwrap_or_default().into(),
        }
    }
}

fn disks(i: InfoReply) -> Vec<Disk> {
    i.disk_info
        .map(|disk_info| disk_info.disks.iter().map(Disk::from).collect())
        .unwrap_or_default()
}

fn cpus(i: InfoReply) -> Vec<Cpu> {
    i.cpu_info
        .map(|cpu_info| cpu_info.cpus.iter().map(Cpu::from).collect())
        .unwrap_or_default()
}
