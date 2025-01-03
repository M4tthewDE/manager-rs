use humansize::DECIMAL;

use crate::proto::MemoryInfo;

pub struct Memory {
    pub total: String,
    pub free: String,
    pub available: String,
    pub used: String,
}

impl From<MemoryInfo> for Memory {
    fn from(m: MemoryInfo) -> Self {
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
