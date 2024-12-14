use cpu::Cpu;
use disk::Disk;
use memory::Memory;

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
