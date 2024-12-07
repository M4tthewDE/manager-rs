use humansize::DECIMAL;

use crate::memory_proto::MemoryReply;

pub struct Memory {
    pub total: String,
    pub free: String,
    pub available: String,
}

impl Memory {
    pub fn new(m: &MemoryReply) -> Self {
        Self {
            total: humansize::format_size(m.total, DECIMAL),
            free: humansize::format_size(m.free, DECIMAL),
            available: humansize::format_size(m.available, DECIMAL),
        }
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            total: "n/a".to_string(),
            free: "n/a".to_string(),
            available: "n/a".to_string(),
        }
    }
}
