use proto::{Cpu, Disk};

pub mod proto {
    tonic::include_proto!("manager");
}

impl From<&sysinfo::Cpu> for Cpu {
    fn from(c: &sysinfo::Cpu) -> Self {
        Self {
            name: c.name().to_string(),
            cpu_usage: c.cpu_usage(),
            frequency: c.frequency(),
        }
    }
}

impl From<&sysinfo::Disk> for Disk {
    fn from(d: &sysinfo::Disk) -> Self {
        Self {
            name: d.name().to_str().unwrap_or_default().to_string(),
            kind: d.kind().to_string(),
            file_system: d.file_system().to_str().unwrap_or_default().to_string(),
            total_space: d.total_space(),
            available_space: d.available_space(),
        }
    }
}
