use cpu::Cpu;
use disk::Disk;
use memory::Memory;

use super::proto::InfoReply;

pub mod cpu;
pub mod disk;
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
}

impl From<&InfoReply> for Info {
    fn from(i: &InfoReply) -> Self {
        let disks = i
            .disk_info
            .clone()
            .map(|disk_info| disk_info.disks.iter().map(Disk::from).collect())
            .unwrap_or_default();
        let memory = Memory::from(&i.memory_info.unwrap_or_default());
        let cpus = i
            .cpu_info
            .clone()
            .map(|cpu_info| cpu_info.cpus.iter().map(Cpu::from).collect())
            .unwrap_or_default();

        Self {
            name: i.name.clone(),
            kernel_version: i.kernel_version.clone(),
            os_version: i.os_version.clone(),
            host_name: i.host_name.clone(),
            memory,
            disks,
            cpus,
        }
    }
}
