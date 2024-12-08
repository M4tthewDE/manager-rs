use crate::state::MemoryReply;
use humansize::DECIMAL;

pub struct Memory {
    pub total: String,
    pub free: String,
    pub available: String,
    pub used: String,
}

impl Memory {
    pub fn new(m: &MemoryReply) -> Self {
        Self {
            total: humansize::format_size(m.total, DECIMAL),
            free: humansize::format_size(m.free, DECIMAL),
            available: humansize::format_size(m.available, DECIMAL),
            used: humansize::format_size(m.used, DECIMAL),
        }
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            total: "n/a".to_string(),
            free: "n/a".to_string(),
            available: "n/a".to_string(),
            used: "n/a".to_string(),
        }
    }
}

pub struct Disk {
    pub name: String,
    pub kind: String,
    pub file_system: String,
    pub total_space: String,
    pub available_space: String,
}

impl Disk {
    pub fn new(d: &super::proto::Disk) -> Self {
        Self {
            name: d.name.clone(),
            kind: d.kind.clone(),
            file_system: d.file_system.clone(),
            total_space: humansize::format_size(d.total_space, DECIMAL),
            available_space: humansize::format_size(d.available_space, DECIMAL),
        }
    }
}

impl Default for Disk {
    fn default() -> Self {
        Self {
            name: "n/a".to_string(),
            kind: "n/a".to_string(),
            file_system: "n/a".to_string(),
            total_space: "n/a".to_string(),
            available_space: "n/a".to_string(),
        }
    }
}
