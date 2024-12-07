use anyhow::{Context, Result};
use humansize::DECIMAL;
use std::path::PathBuf;

use procfs::{FromRead, Meminfo};

pub struct Memory {
    pub total: String,
    pub free: String,
    pub available: String,
}

pub fn calculate_memory() -> Result<Memory> {
    let meminfo = Meminfo::from_file(PathBuf::from("/proc/meminfo"))?;

    Ok(Memory {
        total: humansize::format_size(meminfo.mem_total, DECIMAL),
        free: humansize::format_size(meminfo.mem_free, DECIMAL),
        available: humansize::format_size(
            meminfo
                .mem_available
                .context("no 'available' memory found")?,
            DECIMAL,
        ),
    })
}
